use std::env;

pub struct Config {
    pub database_url: String,
    pub server_addr: String,
    pub db_max_connections: u32,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://user:pass@localhost:5432/ev_db".to_string());

        let server_addr = env::var("SERVER_ADDR").unwrap_or_else(|_| "0.0.0.0:50051".to_string());

        let db_max_connections = env::var("DB_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .unwrap_or(5);

        Self {
            database_url,
            server_addr,
            db_max_connections,
        }
    }
}
