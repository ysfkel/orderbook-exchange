use crate::config::MarketEntry;
use crate::error::ProgramResult;
use crate::repositories::{AssetRepo, MarketRepo, MarketRepository};

pub async fn seed_db(
    markets: &[MarketEntry],
    asset_repo: &AssetRepo,
    market_repo: &MarketRepo,
) -> ProgramResult<()> {
    for m in markets {
        let base = asset_repo.get_or_create(&m.base).await?;
        let quote = asset_repo.get_or_create(&m.quote).await?;
        market_repo.get_or_create(
          base.id,
            quote.id,
            m.market_index,
            m.tick_size_scaled(),
            m.lot_size_scaled(),
            m.min_notional_scaled(),
            m.price_scale(),
            m.qty_scale(),
            true
        ).await?;
    }

    Ok(())
}
