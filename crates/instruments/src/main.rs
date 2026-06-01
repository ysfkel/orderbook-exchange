mod config;
mod db;
mod dto;
mod error;
mod models;
mod repositories;
mod services;

use std::sync::Arc;
use tonic::transport::Server;
use tracing::info;

use services::MarketsService;
use shared_protos::markets::markets_server::MarketsServer;

use crate::db::seed_db;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Logging
    tracing_subscriber::fmt::init();

    // Config
    let config = config::Config::from_env();
    info!("Starting with config: {:?}", config);

    // Database
    db::ensure_database(&config).await;
    let pool = db::create_pool(&config).await;
    db::run_migrations(&pool).await;
    info!("Database ready");

    // Repositories
    let asset_repo = repositories::AssetRepo::new(pool.clone());
    let market_repo = repositories::MarketRepo::new(pool.clone());
    let m = config::load_markets();

    seed_db(&m.markets, &asset_repo, &market_repo)
        .await
        .expect("seed failed");
    info!("db seed completed");

    let addr = "127.0.0.1:50051".parse()?;
    Server::builder()
        .add_service(MarketsServer::new(MarketsService::new(Arc::new(
            market_repo,
        ))))
        .serve(addr)
        .await?;

    Ok(())
}
