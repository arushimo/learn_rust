pub mod domain;
pub mod infrastructure;
pub mod presentation;
pub mod usecase;

use infrastructure::postgres_repository::PostgresChargeSessionRepository;
use presentation::grpc_handler::{
    charging_v1::charging_service_server::ChargingServiceServer, MyChargingService,
};
use usecase::charge_session_usecase::ChargeSessionUsecase;

use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tonic::transport::Server;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    infrastructure::telemetry::init_telemetry();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://user:pass@localhost:5432/ev_db".to_string());

    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("DBに接続できませんでした");

    let repository = PostgresChargeSessionRepository::new(db_pool);
    let repository_arc = Arc::new(repository);

    let usecase = ChargeSessionUsecase::new(repository_arc);
    let usecase_arc = Arc::new(usecase);

    let service = MyChargingService::new(usecase_arc);

    let addr = "0.0.0.0:50051".parse()?;
    info!("gRPC サーバーをポート 50051 で起動します🚀");

    Server::builder()
        .add_service(ChargingServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
