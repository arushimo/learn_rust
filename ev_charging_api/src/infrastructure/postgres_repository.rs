use crate::domain::charge_session::ChargeSession;
use crate::domain::kwh::Kwh;
use crate::domain::repository::ChargeSessionRepository;
use sqlx::PgPool;

pub struct PostgresChargeSessionRepository {
    pool: PgPool,
}

impl PostgresChargeSessionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[tonic::async_trait]
impl ChargeSessionRepository for PostgresChargeSessionRepository {
    async fn save(&self, session: &ChargeSession) -> Result<i32, String> {
        let record = sqlx::query_as::<_, (i32,)>(
            "INSERT INTO charging_sessions (vehicle_model, charged_kwh, start_time) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(&session.vehicle_model)
        .bind(session.charged_kwh.value())
        .bind(session.start_time)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("DBエラー: {:?}", e);
            "Internal Database Error".to_string()
        })?;

        Ok(record.0)
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<ChargeSession>, String> {
        let record = sqlx::query_as::<_, (i32, String, i32, chrono::DateTime<chrono::Utc>)>(
            "SELECT id, vehicle_model, charged_kwh, start_time FROM charging_sessions WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("DBエラー: {:?}", e);
            "Internal Database Error".to_string()
        })?;

        match record {
            Some((id, model, kwh_val, time)) => {
                let kwh = Kwh::try_from(kwh_val).map_err(|e| e.to_string())?;
                Ok(Some(ChargeSession {
                    id: Some(id),
                    vehicle_model: model,
                    charged_kwh: kwh,
                    start_time: time,
                }))
            }
            None => Ok(None),
        }
    }
}
