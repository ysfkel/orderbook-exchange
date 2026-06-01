use std::sync::Arc;

use crate::{
    models::Market,
    repositories::{MarketRepository, Repository},
};
use shared_protos::markets::{
    GetMarketsIdsReply, GetMarketsIdsRequest, GetMarketsReply, GetMarketsRequest,
    Market as MarketProto, MarketId, markets_server::Markets,
};
use tonic::{Request, Response, Status};

pub struct MarketsService<T> {
    repo: Arc<T>,
}

impl<T> MarketsService<T>
where
    T: Send + Sync,
{
    pub fn new(repo: Arc<T>) -> Self {
        Self { repo }
    }
}

#[tonic::async_trait]
impl<T: Repository<Market> + MarketRepository + Send + Sync + 'static> Markets for MarketsService<T>
where
    T: Send + Sync + 'static,
{
    async fn get_markets(
        &self,
        _: Request<GetMarketsRequest>,
    ) -> Result<Response<GetMarketsReply>, Status> {
        let markets = self.repo.find_all_with_assets().await?;

        let mut markets_list = vec![];

        for market in markets {
            let symbol = format!("{}-{}", market.base, market.quote);
            markets_list.push(MarketProto {
                id: market.id,
                market_index: market.market_index,
                symbol,
                is_active: market.is_active,
                price_scale: market.price_scale,
                qty_scale: market.qty_scale,
                tick_size: market.tick_size,
                lot_size: market.lot_size,
                min_notional: market.min_notional,
            });
        }

        let response = GetMarketsReply {
            markets: markets_list,
        };
        Ok(Response::new(response))
    }

    async fn get_markets_ids(
        &self,
        _: Request<GetMarketsIdsRequest>,
    ) -> Result<Response<GetMarketsIdsReply>, Status> {
        let markets = self.repo.find_all().await?;

        let mut markets_list = vec![];

        for market in markets {
            markets_list.push(MarketId {
                id: market.id,
                market_index: market.market_index,
            });
        }

        let response = GetMarketsIdsReply {
            markets_ids: markets_list,
        };
        Ok(Response::new(response))
    }
}
