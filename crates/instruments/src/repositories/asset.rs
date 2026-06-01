use super::Repository;
use crate::{error::ProgramResult, models::Asset};
use async_trait::async_trait;
use sqlx::PgPool;

#[derive(Debug)]
pub struct AssetRepo {
    pub pool: PgPool,
}

#[async_trait]
impl Repository<Asset> for AssetRepo {
    async fn find_all(&self) -> ProgramResult<Vec<Asset>> {
        let assets = sqlx::query_as!(Asset, "SELECT id, name FROM assets")
            .fetch_all(&self.pool)
            .await?;
        Ok(assets)
    }

    async fn find_by_id(&self, id: i32) -> ProgramResult<Option<Asset>> {
        let asset = sqlx::query_as!(Asset, "SELECT id, name FROM assets WHERE id = $1", id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(asset)
    }
}

impl AssetRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, symbol: &str) -> ProgramResult<Asset> {
        let asset = sqlx::query_as!(
            Asset,
            "INSERT INTO assets (name) VALUES ($1) RETURNING id, name",
            symbol
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(asset)
    }

    pub async fn get_or_create(&self, symbol: &str) -> ProgramResult<Asset> {
        let asset = sqlx::query_as!(
            Asset,
            "INSERT INTO assets (name) VALUES ($1)
          -- if a row with this name already exists...
         ON CONFLICT (name) 
          -- ...do a pointless update (same value) just to force a RETURNING
         DO UPDATE SET name = EXCLUDED.name
          -- EXCLUDED.name = the value we tried to insert ($1)
          -- this makes postgres ALWAYS return the row, whether inserted or not
         RETURNING id, name",
            symbol
        )
        // safe to use fetch_one because we always get exactly one row back:
        // - fresh insert    → returns the new row
        // - conflict/exists → returns the existing row (after the no-op update)
        .fetch_one(&self.pool)
        .await?;
        Ok(asset)
    }
}
