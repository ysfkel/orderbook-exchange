use std::path::Path;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub max_connections: u32,
}

impl Config {
    pub fn from_env() -> Self {
        let env_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(".env");
        dotenv::from_path(&env_path).ok();
        Self {
            database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            max_connections: std::env::var("DB_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .expect("DB_MAX_CONNECTIONS must be a number"),
        }
    }
}
