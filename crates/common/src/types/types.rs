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

pub const MAX_OFFER_RETRIES: u8 = 3;