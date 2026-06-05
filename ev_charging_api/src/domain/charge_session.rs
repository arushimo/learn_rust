use crate::domain::kwh::Kwh;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct ChargeSession {
    pub id: Option<i32>,
    pub vehicle_model: String,
    pub charged_kwh: Kwh,
    pub start_time: DateTime<Utc>,
}

impl ChargeSession {
    pub fn new(vehicle_model: String, charged_kwh: Kwh, start_time: DateTime<Utc>) -> Self {
        Self {
            id: None,
            vehicle_model,
            charged_kwh,
            start_time,
        }
    }
}
