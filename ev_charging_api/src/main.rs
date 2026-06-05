// 💡 OpenTelemetry 関連の正しいインポート（WithExportConfigを復活させます）
use chrono::{DateTime, Utc};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tonic::{transport::Server, Request, Response, Status};
use tracing::{info, instrument};

use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig; // 👈 with_endpoint を使うために必須
use opentelemetry_sdk::trace::{Config, Tracer};

// ==========================================
// 1. 自動生成された gRPC コードの取り込み
// ==========================================
// protoのパッケージ名「charging」に対応するモジュールを定義
pub mod charging {
    // build.rs が生成したコードをこの場所（インライン）に展開するマクロ
    tonic::include_proto!("charging");
}

// 扱いやすいように自動生成されたサービスや構造体をインポート
use charging::charging_service_server::{ChargingService, ChargingServiceServer};
use charging::{ChargeSessionRequest, ChargeSessionResponse, GetSessionRequest};

// ==========================================
// 2. ドメインモデリング (第16回：Newtypeパターン)
// ==========================================
#[derive(Debug, Clone)]
pub struct Kwh(i32);

impl TryFrom<i32> for Kwh {
    type Error = &'static str;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if value > 0 && value <= 150 {
            Ok(Kwh(value))
        } else {
            Err("無効な充電量です。1〜150kWhの間で指定してください。")
        }
    }
}

// ==========================================
// 3. サービスの実装（ハンドラ構造体）
// ==========================================
// 💡 AxumのStateの代わりに、構造体のフィールドにDBプール（DI）を持たせます
pub struct MyChargingService {
    db: PgPool,
}

// Tonicが要求するトレイト（インターフェース）を構造体に実装する
#[tonic::async_trait]
impl ChargingService for MyChargingService {

    // 💡 ① セッションの作成 (POSTの代わり)
    #[instrument(skip(self))] // 引数のself(DBプール)を除外してJaegerに計装
    async fn create_session(
        &self,
        request: Request<ChargeSessionRequest>,
    ) -> Result<Response<ChargeSessionResponse>, Status> {
        info!("gRPC: CreateSession リクエストを受信しました");
        
        // request.into_inner() で、gRPCのガワを剥いて中身の構造体を取り出す
        let req = request.into_inner();

        // Newtypeバリデーション (エラー時は gRPC の INVALID_ARGUMENT ステータスを返す)
        let kwh = Kwh::try_from(req.charged_kwh)
            .map_err(|e| Status::invalid_argument(e))?;

        // 文字列で送られてきた時間を Chrono でパース (第18回：タイムゾーンの明示)
        let parsed_time = DateTime::parse_from_rfc3339(&req.start_time)
            .map_err(|_| Status::invalid_argument("start_time のフォーマットが不正です（RFC3339を期待）"))?
            .with_timezone(&Utc); // UTCに型をカチッと固定

        // SQLxによるDB挿入
        let record = sqlx::query_as::<_, (i32,)> (
            "INSERT INTO charging_sessions (vehicle_model, charged_kwh, start_time) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(&req.vehicle_model)
        .bind(kwh.0)
        .bind(parsed_time)
        .fetch_one(&self.db)
        .await
        .map_err(|e| {
            tracing::error!("DBエラー: {:?}", e);
            Status::internal("Internal Database Error")
        })?;

        // レスポンスの組み立て
        let reply = ChargeSessionResponse {
            id: record.0,
            vehicle_model: req.vehicle_model,
            charged_kwh: kwh.0,
            start_time: parsed_time.to_rfc3339(),
        };

        Ok(Response::new(reply))
    }

    // 💡 ② セッションの取得 (GETの代わり)
    #[instrument(skip(self))]
    async fn get_session(
        &self,
        request: Request<GetSessionRequest>,
    ) -> Result<Response<ChargeSessionResponse>, Status> {
        let req = request.into_inner();
        info!("gRPC: GetSession リクエストを受信しました (ID: {})", req.id);

        let record = sqlx::query_as::<_, (i32, String, i32, DateTime<Utc>)>(
            "SELECT id, vehicle_model, charged_kwh, start_time FROM charging_sessions WHERE id = $1"
        )
        .bind(req.id)
        .fetch_optional(&self.db)
        .await
        .map_err(|_| Status::internal("Internal Database Error"))?;

        match record {
            Some((id, model, kwh, time)) => {
                let reply = ChargeSessionResponse {
                    id,
                    vehicle_model: model,
                    charged_kwh: kwh,
                    start_time: time.to_rfc3339(),
                };
                Ok(Response::new(reply))
            }
            None => Err(Status::not_found(format!("ID: {} のセッションは見つかりません", req.id))),
        }
    }
}

// ==========================================
// 4. OpenTelemetry (Jaeger) とメイン関数
fn init_tracer() -> Tracer {
    let provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://jaeger:4317"),
        )
        .with_trace_config(
            Config::default() // 👈 古い config() ではなく Config::default() を使用
                .with_resource(opentelemetry_sdk::Resource::new(vec![
                    opentelemetry::KeyValue::new("service.name", "ev_charging_grpc_api"),
                ])),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .expect("Failed to initialize tracer");

    provider.tracer("ev_charging_grpc_api")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // .env ファイルの読み込み
    dotenvy::dotenv().ok();

    // トレース・ログの初期化
    let tracer = init_tracer();
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    
    use tracing_subscriber::prelude::*;
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .with(telemetry)
        .init();

    // DB接続
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://user:pass@localhost:5432/ev_db".to_string());
        // 💡 本番環境に特化させる場合の、最も厳格な書き方
        // .expect("環境変数 DATABASE_URL が設定されていません。起動を中止します。");
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("DBに接続できませんでした");

    // サービスのインスタンス作成
    let service = MyChargingService { db: db_pool };

    // gRPC サーバーの起動設定
    let addr = "0.0.0.0:50051".parse()?; // gRPCの標準的なポートに変更
    info!("gRPC サーバーをポート 50051 で起動します🚀");

    Server::builder()
        .add_service(ChargingServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
