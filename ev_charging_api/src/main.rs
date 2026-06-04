use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Arc;
use tracing::{info, instrument};
use opentelemetry_sdk::trace::Tracer;

// ==========================================
// 1. ドメインモデリング (Newtype & Serde)
// ==========================================

// 充電量(kWh)を表すNewtype。マイナス値などを型レベルで防ぐ
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Kwh(i32);

impl TryFrom<i32> for Kwh {
    type Error = &'static str;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value > 0 && value <= 150 { // バッテリー容量の現実的な上限を想定
            Ok(Kwh(value))
        } else {
            Err("無効な充電量です")
        }
    }
}

// リクエスト/レスポンス用構造体
#[derive(Debug, Serialize, Deserialize)]
pub struct ChargeSession {
    pub id: Option<i32>, // 登録時は不要なのでOption
    pub vehicle_model: String, // 例: "Model Y", "e-Vitara"
    pub charged_kwh: i32,      // APIリクエスト時は一旦i32で受ける
    pub start_time: DateTime<Utc>, // Chronoでタイムゾーンを強制
}

// アプリケーションの状態 (DI用)
pub struct AppState {
    pub db: PgPool,
}

// ==========================================
// 2. OpenTelemetry (Jaeger) セットアップ
// ==========================================
fn init_tracer() -> Tracer {
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .with_trace_config(
            opentelemetry_sdk::trace::config()
                .with_resource(opentelemetry_sdk::Resource::new(vec![
                    opentelemetry::KeyValue::new("service.name", "ev_charging_api"),
                ])),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .expect("Failed to initialize tracer")
}

// ==========================================
// 3. APIハンドラ (Axum & SQLx & Tracing)
// ==========================================

// 💡 instrumentマクロで、関数の引数(payload)を自動的にJaegerへ送信する
#[instrument(skip(state))]
async fn create_session(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ChargeSession>,
) -> Result<(StatusCode, Json<ChargeSession>), (StatusCode, String)> {
    info!("新しい充電セッションの記録リクエストを受信しました");

    // Newtypeによるドメインバリデーション
    let kwh = Kwh::try_from(payload.charged_kwh)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    // SQLxによるDB挿入 (query_as を使用)
    let record = sqlx::query_as::<_, (i32,)>(
        r#"INSERT INTO charging_sessions (vehicle_model, charged_kwh, start_time) 
           VALUES ($1, $2, $3) RETURNING id"#
    )
    .bind(&payload.vehicle_model)
    .bind(kwh.0) // 検証済みの値を取り出す
    .bind(payload.start_time)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("DBエラー: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, "DB Error".to_string())
    })?;

    let mut response = payload;
    response.id = Some(record.0);

    info!("充電セッションを保存しました (ID: {})", record.0);
    Ok((StatusCode::CREATED, Json(response)))
}

#[instrument(skip(state))]
async fn get_session(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> Result<Json<ChargeSession>, (StatusCode, String)> {
    // 💡 SQLxのマクロを使わず実行時にマッピングするアプローチ
    // (マクロ `query_as!` はコンパイル時にDB起動が必須になるため、Docker環境でのビルドを簡単にするためにこちらを採用)
    let record = sqlx::query_as::<_, (i32, String, i32, DateTime<Utc>)>(
        "SELECT id, vehicle_model, charged_kwh, start_time FROM charging_sessions WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "DB Error".to_string()))?;

    match record {
        Some((id, model, kwh, time)) => Ok(Json(ChargeSession {
            id: Some(id),
            vehicle_model: model,
            charged_kwh: kwh,
            start_time: time,
        })),
        None => Err((StatusCode::NOT_FOUND, "Not Found".to_string())),
    }
}

// ==========================================
// 4. メイン関数
// ==========================================
#[tokio::main]
async fn main() {
    // Tracerの初期化とSubscriberの登録
    let tracer = init_tracer();
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    
    use tracing_subscriber::prelude::*;
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer()) // 標準出力用
        .with(telemetry)                        // Jaeger送信レイヤー
        .init();

    // DB接続プールの作成
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/ev_db".to_string());
    
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("DBに接続できませんでした");

    let app_state = Arc::new(AppState { db: db_pool });

    // Axumルーターの構築とDI(State)
    let app = Router::new()
        .route("/sessions", post(create_session))
        .route("/sessions/:id", get(get_session))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("サーバーをポート3000で起動します🚀");
    axum::serve(listener, app).await.unwrap();
}
