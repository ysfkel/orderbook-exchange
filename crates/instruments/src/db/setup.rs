use crate::config::Config;
use sqlx::migrate::MigrateDatabase;
use sqlx::{PgPool, Postgres, postgres::PgPoolOptions};

pub async fn ensure_database(config: &Config) {
    if !Postgres::database_exists(&config.database_url)
        .await
        .expect("Failed to check database existence")
    {
        Postgres::create_database(&config.database_url)
            .await
            .expect("Failed to create database");
    }
}

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
