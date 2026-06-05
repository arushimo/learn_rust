use chrono::Utc;

// 💡 テストディレクトリ（外部クレート扱い）からもgRPCの型にアクセスするため、
// ここでもう一度自動生成コードを展開します。
pub mod charging {
    tonic::include_proto!("charging");
}

// 自動生成されたgRPCクライアントと、リクエスト用の構造体をインポート
use charging::charging_service_client::ChargingServiceClient;
use charging::{ChargeSessionRequest, GetSessionRequest};

#[tokio::test]
async fn test_create_and_fetch_session() {
    // 💡 1. クライアントの作成（Port 50051 の gRPC サーバーへ接続）
    // ※テスト実行時は、裏でサーバー（cargo run または Docker）が起動している必要があります
    let mut client = ChargingServiceClient::connect("http://localhost:50051")
        .await
        .expect("gRPCサーバーへの接続に失敗しました。サーバーが起動しているか確認してください。");

    // ===================================================
    // 2. セッションの作成 (POSTの代わり)
    // ===================================================
    let create_req = tonic::Request::new(ChargeSessionRequest {
        vehicle_model: "Tesla Model Y".to_string(),
        charged_kwh: 65,
        start_time: Utc::now().to_rfc3339(),
    });

    let post_resp = client
        .create_session(create_req)
        .await
        .expect("CreateSession リクエストに失敗しました")
        .into_inner(); // gRPCのレスポンスのガワ（メタデータ等）を剥がして中身を取り出す

    // 自動採番されたIDを取得
    let session_id = post_resp.id;

    // ===================================================
    // 3. 作成したセッションの取得 (GETの代わり)
    // ===================================================
    let get_req = tonic::Request::new(GetSessionRequest { id: session_id });

    let get_resp = client
        .get_session(get_req)
        .await
        .expect("GetSession リクエストに失敗しました")
        .into_inner();

    // 💡 4. 検証（アサーション）
    assert_eq!(get_resp.vehicle_model, "Tesla Model Y");
    assert_eq!(get_resp.charged_kwh, 65);

    // （オプション）新しく追加したバリデーションの異常系テスト
    let invalid_req = tonic::Request::new(ChargeSessionRequest {
        vehicle_model: "Error Car".to_string(),
        charged_kwh: 999, // 150kWhオーバー
        start_time: Utc::now().to_rfc3339(),
    });
    let err_resp = client.create_session(invalid_req).await;
    assert!(err_resp.is_err(), "異常値はエラーとして弾かれるべきです");
}
