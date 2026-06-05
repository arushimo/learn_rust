use crate::usecase::charge_session_usecase::ChargeSessionUsecase;
use chrono::{TimeZone, Utc};
use prost_types::Timestamp;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{info, instrument};

pub mod charging_v1 {
    tonic::include_proto!("charging.v1");
}

use charging_v1::charging_service_server::ChargingService;
use charging_v1::{ChargeSession, CreateChargeSessionRequest, GetChargeSessionRequest};

pub struct MyChargingService {
    usecase: Arc<ChargeSessionUsecase>,
}

impl MyChargingService {
    pub fn new(usecase: Arc<ChargeSessionUsecase>) -> Self {
        Self { usecase }
    }
}

#[tonic::async_trait]
impl ChargingService for MyChargingService {
    #[instrument(skip(self))]
    async fn create_charge_session(
        &self,
        request: Request<CreateChargeSessionRequest>,
    ) -> Result<Response<ChargeSession>, Status> {
        info!("gRPC: CreateChargeSession リクエストを受信しました");

        let req = request.into_inner();
        let session = req
            .charge_session
            .ok_or_else(|| Status::invalid_argument("charge_session が指定されていません"))?;

        let start_time_ts = session
            .start_time
            .ok_or_else(|| Status::invalid_argument("start_time が指定されていません"))?;
        let parsed_time = Utc
            .timestamp_opt(start_time_ts.seconds, start_time_ts.nanos as u32)
            .single()
            .ok_or_else(|| Status::invalid_argument("無効な start_time です"))?;

        let created_session = self
            .usecase
            .create_session(session.vehicle_model, session.charged_kwh, parsed_time)
            .await
            .map_err(|e| {
                if e.contains("無効な充電量") {
                    Status::invalid_argument(e)
                } else {
                    Status::internal(e)
                }
            })?;

        let id = created_session.id.unwrap();
        let reply = ChargeSession {
            name: format!("chargeSessions/{}", id),
            vehicle_model: created_session.vehicle_model,
            charged_kwh: created_session.charged_kwh.value(),
            start_time: Some(start_time_ts),
        };

        Ok(Response::new(reply))
    }

    #[instrument(skip(self))]
    async fn get_charge_session(
        &self,
        request: Request<GetChargeSessionRequest>,
    ) -> Result<Response<ChargeSession>, Status> {
        let req = request.into_inner();
        info!(
            "gRPC: GetChargeSession リクエストを受信しました (Name: {})",
            req.name
        );

        let id_str = req.name.strip_prefix("chargeSessions/").ok_or_else(|| {
            Status::invalid_argument(
                "無効なリソース名です。'chargeSessions/{id}' の形式で指定してください。",
            )
        })?;
        let id: i32 = id_str.parse().map_err(|_| {
            Status::invalid_argument("リソース名のID部分は整数である必要があります。")
        })?;

        let session_opt = self
            .usecase
            .get_session(id)
            .await
            .map_err(Status::internal)?;

        match session_opt {
            Some(session) => {
                let time = session.start_time;
                let ts = Timestamp {
                    seconds: time.timestamp(),
                    nanos: time.timestamp_subsec_nanos() as i32,
                };

                let reply = ChargeSession {
                    name: format!("chargeSessions/{}", session.id.unwrap()),
                    vehicle_model: session.vehicle_model,
                    charged_kwh: session.charged_kwh.value(),
                    start_time: Some(ts),
                };
                Ok(Response::new(reply))
            }
            None => Err(Status::not_found(format!(
                "リソース {} は見つかりません",
                req.name
            ))),
        }
    }
}
