use crate::domain::charge_session::ChargeSession;

#[tonic::async_trait]
pub trait ChargeSessionRepository: Send + Sync {
    async fn save(&self, session: &ChargeSession) -> Result<i32, String>;
    async fn find_by_id(&self, id: i32) -> Result<Option<ChargeSession>, String>;
}
