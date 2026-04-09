use serde::Deserialize;
use super::market_entry::MarketEntry;

#[derive(Deserialize)]
pub struct MarketsConfig {
    pub markets: Vec<MarketEntry>,
}

pub fn load_markets() -> MarketsConfig {
    let markets: MarketsConfig =
        toml::from_str(include_str!("../../markets.toml")).expect("failed to load markets config");
    
    if !validate_markets_config(&markets) {
        panic!("Invalid markets configuration");
    }

    markets
}

fn validate_markets_config(config: &MarketsConfig) -> bool {

    let mut seen_ticker = std::collections::HashSet::new();

    for (i, market) in config.markets.iter().enumerate() {
     
        if market.base.trim().is_empty() {
            eprintln!("Market at index {} has an empty base asset", i);
            return false;
        }

        if market.quote.trim().is_empty() {
            eprintln!("Market at index {} has an empty quote asset", i);
            return false;
        }

        if market.base == market.quote {
            eprintln!("Market at index {} has the same base and quote asset", i);
            return false;
        }

        if !seen_ticker.insert((market.base.clone(), market.quote.clone())) {
            eprintln!("Duplicate market found: {}-{}", market.base, market.quote);
            return false;
        }

        if i != market.market_index as usize {
            eprintln!(
                "Market at index {} has a market_index of {}, which does not match its position",
                i, market.market_index
            );
            return false;
        }
    }

    true
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_load_markets() {
        let markets = load_markets();
        assert_eq!(markets.markets.len(), 2);
    }
}
