use crate::domain::charge_session::ChargeSession;
use crate::domain::kwh::Kwh;
use crate::domain::repository::ChargeSessionRepository;
use chrono::{DateTime, Utc};
use std::sync::Arc;

pub struct ChargeSessionUsecase {
    repository: Arc<dyn ChargeSessionRepository>,
}

impl ChargeSessionUsecase {
    pub fn new(repository: Arc<dyn ChargeSessionRepository>) -> Self {
        Self { repository }
    }

    pub async fn create_session(
        &self,
        vehicle_model: String,
        charged_kwh: i32,
        start_time: DateTime<Utc>,
    ) -> Result<ChargeSession, String> {
        let kwh = Kwh::try_from(charged_kwh)?;
        let mut session = ChargeSession::new(vehicle_model, kwh, start_time);

        let id = self.repository.save(&session).await?;
        session.id = Some(id);

        Ok(session)
    }

    pub async fn get_session(&self, id: i32) -> Result<Option<ChargeSession>, String> {
        self.repository.find_by_id(id).await
    }
}
