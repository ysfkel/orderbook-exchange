use crate::config::Config;
use sqlx::{PgPool, postgres::PgPoolOptions};

pub async fn create_pool(config: &Config) -> PgPool {
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.database_url)
        .await
        .expect("Failed to create database pool")
}

pub async fn run_migrations(pool: &PgPool) {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .expect("Failed to run migrations")
}
