use chrono::Utc;
use prost_types::Timestamp;

// 💡 テストディレクトリ（外部クレート扱い）からもgRPCの型にアクセスするため、
// ここでもう一度自動生成コードを展開します。
pub mod charging_v1 {
    tonic::include_proto!("charging.v1");
}

// 自動生成されたgRPCクライアントと、リクエスト用の構造体をインポート
use charging_v1::charging_service_client::ChargingServiceClient;
use charging_v1::{ChargeSession, CreateChargeSessionRequest, GetChargeSessionRequest};

#[tokio::test]
async fn test_create_and_fetch_session() {
    // 💡 1. クライアントの作成
    let mut client = ChargingServiceClient::connect("http://localhost:50051")
        .await
        .expect("gRPCサーバーへの接続に失敗しました。サーバーが起動しているか確認してください。");

    let now = Utc::now();
    let ts = Timestamp {
        seconds: now.timestamp(),
        nanos: now.timestamp_subsec_nanos() as i32,
    };

    // ===================================================
    // 2. セッションの作成
    // ===================================================
    let create_req = tonic::Request::new(CreateChargeSessionRequest {
        parent: "".to_string(),
        charge_session_id: "".to_string(),
        charge_session: Some(ChargeSession {
            name: "".to_string(),
            vehicle_model: "Tesla Model Y".to_string(),
            charged_kwh: 65,
            start_time: Some(ts),
        }),
    });

    let post_resp = client
        .create_charge_session(create_req)
        .await
        .expect("CreateChargeSession リクエストに失敗しました")
        .into_inner();

    // 自動採番された名前を取得 (e.g. "chargeSessions/1")
    let session_name = post_resp.name;

    // ===================================================
    // 3. 作成したセッションの取得
    // ===================================================
    let get_req = tonic::Request::new(GetChargeSessionRequest { name: session_name });

    let get_resp = client
        .get_charge_session(get_req)
        .await
        .expect("GetChargeSession リクエストに失敗しました")
        .into_inner();

    // 💡 4. 検証（アサーション）
    assert_eq!(get_resp.vehicle_model, "Tesla Model Y");
    assert_eq!(get_resp.charged_kwh, 65);

    // （オプション）新しく追加したバリデーションの異常系テスト
    let invalid_req = tonic::Request::new(CreateChargeSessionRequest {
        parent: "".to_string(),
        charge_session_id: "".to_string(),
        charge_session: Some(ChargeSession {
            name: "".to_string(),
            vehicle_model: "Error Car".to_string(),
            charged_kwh: 999, // 150kWhオーバー
            start_time: Some(ts),
        }),
    });
    let err_resp = client.create_charge_session(invalid_req).await;
    assert!(err_resp.is_err(), "異常値はエラーとして弾かれるべきです");
}
