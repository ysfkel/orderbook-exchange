use shared_protos::markets::markets_client::MarketsClient;
use shared_protos::markets::{GetMarketsRequest, Market};
use tonic::transport::Channel;
use tracing::info;

use crate::error::ProgramError;

pub struct InstrumentsClient {
    client: MarketsClient<Channel>,
}

impl InstrumentsClient {
    pub async fn connect(url: &str) -> Result<Self, ProgramError> {
        let client = MarketsClient::connect(url.to_owned()).await?;
        info!("connected to instruments service at {}", url);
        Ok(Self { client })
    }

    pub async fn get_instruments(&mut self) -> Result<Vec<Market>, ProgramError> {
        let response = self.client.get_markets(GetMarketsRequest {}).await?;
        Ok(response.into_inner().markets)
    }
}
