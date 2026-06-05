pub mod config;
pub mod domain;
pub mod infrastructure;
pub mod presentation;
pub mod usecase;

use config::Config;
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
    let config = Config::from_env();

    infrastructure::telemetry::init_telemetry();

    let db_pool = PgPoolOptions::new()
        .max_connections(config.db_max_connections)
        .connect(&config.database_url)
        .await
        .expect("DBに接続できませんでした");

    let repository = Arc::new(PostgresChargeSessionRepository::new(db_pool));
    let usecase = Arc::new(ChargeSessionUsecase::new(repository));
    let service = MyChargingService::new(usecase);

    let addr = config.server_addr.parse()?;
    info!("gRPC サーバーを {} で起動します🚀", config.server_addr);

    Server::builder()
        .add_service(ChargingServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
