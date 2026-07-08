use common::{
    mem_pool::POOL_IDX_NULL,
    types::{ME_MAX_PRICE_LEVELS, OrderSide, PRICE_INVALID, PoolIdx, Price},
};
use zerocopy::{FromBytes, FromZeros, Immutable, IntoBytes, KnownLayout, TryFromBytes};

/// Holds all the orders entered at the same price, arranged in the FIFO priority order.
/// This is achieved by creating a singly linked list of Order objects, arranged
/// in order of highest to lowest priority. For that, we create a head_order_index variable
/// representing the index of the Order , which will represent the first order at this price level,
/// and the other orders following it are chained together in the FIFO order.”
///
/// “The OrdersAtPrice structure also has two pointers to the OrdersAtPrice objects, one for the
/// previous (prev_entry_) and one for the next (next_entry_). This is because the structure
/// itself is a node in a doubly linked list of OrdersAtPrice objects.
/// The doubly linked list of OrdersAtPrice is arranged from the most
/// aggressive to the least aggressive prices on the buy and sell sides.”
#[derive(Debug, IntoBytes, Immutable, KnownLayout, TryFromBytes)]
#[repr(C)]
pub struct OrdersAtPrice {
    /// Price of this price level
    pub price: Price, // offset 0
    /// the head order of this price level.
    /// Highest-priority (oldest) order
    /// at this level; its `prev_order`
    /// is the
    /// tail of the FIFO because the
    /// list is circular.
    pub head_order: PoolIdx, //offset 16
    pub prev_order: PoolIdx, //offset 20
    pub next_order: PoolIdx, //offset 24
    /// side of this price level
    pub side: OrderSide, //offset 28
    _padding: [u8; 3],       //offset 29 + 3 = 32 which is a multiple of 16
}

impl Default for OrdersAtPrice {
    fn default() -> Self {
        Self {
            price: PRICE_INVALID,
            head_order: POOL_IDX_NULL,
            prev_order: POOL_IDX_NULL,
            next_order: POOL_IDX_NULL,
            side: OrderSide::Unset,
            _padding: [0u8; 3],
        }
    }
}

/// The price index comes from price_to_index(price),
/// which converts a raw tick price into an array slot.
/// Each slot holds a PoolIdx — a handle into MemPool<MEOrdersAtPrice>.
///
/// example:
///   |null|null| 7 |null| 3 |null|
///      0   1    2   3    4   5
///   price 102 → index 2 → slot holds PoolIdx(7)
///   then we use the orders_at_price_pool f orderbook.rs
///   orders_at_price_pool[7] = the MEOrdersAtPrice for price
/// price
/// → price_to_index(price)           O(1) arithmetic
/// → OrdersAtPrice[index]     O(1) array read  → PoolIdx
/// → orders_at_price_pool[PoolIdx]   O(1) array read  → &MEOrdersAtPrice
/// → .first_me_order                 O(1)             → PoolIdx into order_pool
/// → order_pool[PoolIdx]             O(1)             → &MEOrder
pub struct PriceOrderAtPrice {
    pub price_orders_at_price: Vec<PoolIdx>,
}

impl PriceOrderAtPrice {
    pub fn new() -> Self {
        Self {
            price_orders_at_price: vec![POOL_IDX_NULL; ME_MAX_PRICE_LEVELS],
        }
    }

    #[inline]
    pub fn get_orders_at_price(&self, price_index: usize) -> PoolIdx {
        self.price_orders_at_price[price_index]
    }
}
