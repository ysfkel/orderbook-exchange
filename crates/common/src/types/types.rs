use zerocopy::{FromZeros, Immutable, IntoBytes, KnownLayout, TryFromBytes, Unaligned};

pub type OrderId = u64;
pub type UserId = u32;
pub type ClientId = u32;
pub type Price = u128;
pub type TickerId = u32;
pub type Qty = u128;
pub type Priority = u64;
pub type PoolIdx = u32;

pub const ORDER_ID_INVALID: OrderId = OrderId::MAX;
pub const TICKER_ID_INVALID: TickerId = TickerId::MAX;
pub const CLIENT_ID_INVALID: ClientId = ClientId::MAX;
pub const PRICE_INVALID: Price = Price::MAX;
pub const QTY_INVALID: Qty = Qty::MAX;
pub const PRIORITY_INVALID: Priority = Priority::MAX;

// ---------------------------------------------------------------------------
// Limits (the book's ME_MAX_* constants from common/types.h).
// The book uses much larger values (e.g. ME_MAX_ORDER_IDS = 1024 * 1024,
// ME_MAX_NUM_CLIENTS = 256), which pre-allocates gigabytes for the
// ClientOrderHashMap. We use demo-sized values here; scale them up to match
// the book if you have the RAM.
// ---------------------------------------------------------------------------
pub const ME_MAX_TICKERS: usize = 8;
pub const ME_MAX_CLIENT_UPDATES: usize = 256 * 1024;
pub const ME_MAX_MARKET_UPDATES: usize = 256 * 1024;
pub const ME_MAX_NUM_CLIENTS: usize = 64;
pub const ME_MAX_ORDER_IDS: usize = 16 * 1024;
pub const ME_MAX_PRICE_LEVELS: usize = 256;

// type OrderIdInner = [u8; 16];
// type UserIdInner = [u8; 32];

// #[derive(Debug, Clone, Copy, IntoBytes, Unaligned, Immutable, KnownLayout, TryFromBytes)]
// #[repr(transparent)] // This makes OrderId identical to [u8; 16] in memory
// pub struct OrderId(OrderIdInner);

// impl TryFrom<&[u8]> for OrderId {
//     type Error = &'static str;

//     fn try_from(s: &[u8]) -> Result<Self, Self::Error> {
//         if s.len() > 16 {
//             return Err("OrderId cannot exceed 16 bytes");
//         }
//         let mut arr: OrderIdInner = [0u8; 16];
//         arr[..s.len()].copy_from_slice(s);

//         Ok(OrderId(arr))
//     }
// }

// impl Into<[u8; 16]> for OrderId {
//     fn into(self) -> OrderIdInner {
//         self.0
//     }
// }

// impl OrderId {
//     pub fn as_bytes(&self) -> &OrderIdInner {
//         &self.0
//     }
// }

// #[derive(Debug, Clone, Copy, IntoBytes, Unaligned, Immutable, KnownLayout, TryFromBytes)]
// #[repr(transparent)] // This makes OrderId identical to [u8; 16] in memory
// pub struct UserId(UserIdInner);

// impl TryFrom<&[u8]> for UserId {
//     type Error = &'static str;

//     fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
//         if bytes.len() > 32 {
//             return Err("UserId not be greater than 32 bytes");
//         }

//         let mut arr: UserIdInner = [0u8; 32];
//         arr[..bytes.len()].copy_from_slice(bytes);

//         Ok(UserId(arr))
//     }
// }

// impl Into<UserIdInner> for UserId {
//     fn into(self) -> UserIdInner {
//         self.0
//     }
// }

// #[derive(Debug, IntoBytes, Immutable, KnownLayout, TryFromBytes)]
// #[repr(u8)]
// pub enum Asset {
//     USDC,
//     USDT,
//     BTC,
//     ETH,
//     SOL,
// }

// impl Asset {
//     pub fn from_str(asset_str: &str) -> Result<Asset, &'static str> {
//         let asset = match asset_str {
//             "USDC" => Asset::USDC,
//             "USDT" => Asset::USDT,
//             "BTC" => Asset::BTC,
//             "ETH" => Asset::ETH,
//             "SOL" => Asset::SOL,
//             _ => return Err("unsupported asset"),
//         };
//         Ok(asset)
//     }

//     pub fn as_str(&self) -> &str {
//         let asset = match self {
//             Asset::USDC => "USDC",
//             Asset::USDT => "USDT",
//             Asset::BTC => "BTC",
//             Asset::ETH => "ETH",
//             Asset::SOL => "SOL",
//         };
//         asset
//     }
// }

// #[derive(Debug, IntoBytes, Immutable, KnownLayout, TryFromBytes)]
// #[repr(C)]
// pub struct Market(Asset, Asset);

// impl Market {
//     pub fn to_string(&self) -> String {
//         format!("{}-{}", self.0.as_str(), self.1.as_str())
//     }

//     pub fn base(&self) -> &Asset {
//         &self.0
//     }

//     pub fn quote(&self) -> &Asset {
//         &self.1
//     }
// }

// impl std::str::FromStr for Market {
//     type Err = &'static str;

//     // also implements parse for market eg "BTC-USDC".parse()
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         let (base, quote) = s.split_once("-").ok_or("invalid market format")?;

//         Ok(Market(Asset::from_str(base)?, Asset::from_str(quote)?))
//     }
// }
