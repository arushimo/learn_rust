use chrono::Utc;
use serde_json::json;

#[tokio::test]
async fn test_create_and_fetch_session() {
    let client = reqwest::Client::new();
    let base_url = "http://localhost:3000";

    // 1. セッションの作成 (POST)
    let payload = json!({
        "vehicle_model": "Tesla Model Y",
        "charged_kwh": 65,
        "start_time": Utc::now().to_rfc3339()
    });

    let post_resp = client
        .post(&format!("{}/sessions", base_url))
        .json(&payload)
        .send()
        .await
        .expect("POSTリクエストに失敗しました");

    assert_eq!(post_resp.status(), 201);

    // レスポンスから自動採番されたIDを取得
    let created_session: serde_json::Value = post_resp.json().await.unwrap();
    let session_id = created_session["id"].as_i64().unwrap();

    // 2. 作成したセッションの取得 (GET)
    let get_resp = client
        .get(&format!("{}/sessions/{}", base_url, session_id))
        .send()
        .await
        .expect("GETリクエストに失敗しました");

    assert_eq!(get_resp.status(), 200);

    let fetched_session: serde_json::Value = get_resp.json().await.unwrap();
    assert_eq!(fetched_session["vehicle_model"], "Tesla Model Y");
    assert_eq!(fetched_session["charged_kwh"], 65);
}
