use common::types::models::{OrderSide, OrderStatus, OrderType};

// ── bounds ────────────────────────────────────────────────────────────────────
// These are compile-time constants. The entire pool is one contiguous block
// allocated once at startup — no further heap use on the hot path.
pub const MAX_CLIENTS: usize = 256;
pub const MAX_ORDERS_PER_CLIENT: usize = 1024; // must be a power of 2

// ── order state ───────────────────────────────────────────────────────────────
// Mirrors the book's OMOrderState. The OMS tracks these transitions:
//
//   PendingNew  →  Live          (engine replied ACCEPTED)
//   Live        →  PendingCancel (client sent cancel request)
//   PendingCancel → Dead         (engine replied CANCELED)
//   Live        →  Dead          (engine replied FILLED or REJECTED)
//   Dead                         (slot is free and can be reused)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OrderState {
    Dead = 0, // default — slot is free
    PendingNew,
    Live,
    PendingCancel,
}

// ── the order stored in the pool ──────────────────────────────────────────────
// This is the OMS's internal view of an order, not a wire struct.
// No zerocopy derives needed — it never goes on the wire.
#[derive(Debug, Clone, Copy)]
pub struct Order {
    pub client_id: u32,
    pub client_order_id: u32, // the ID the client chose
    pub market_order_id: u32, // the ID we sent to the engine (encodes slot)
    pub market_index: u32,    // which market
    pub price: u128,
    pub quantity: u128,
    pub leaves_qty: u128, // remaining, decremented on partial fills
    pub side: OrderSide,
    pub order_type: OrderType,
    pub state: OrderState,
}

impl Order {
    // A dead/empty slot. Used to initialise the pool.
    const DEAD: Self = Self {
        client_id: 0,
        client_order_id: 0,
        market_order_id: 0,
        market_index: 0,
        price: 0,
        quantity: 0,
        leaves_qty: 0,
        side: OrderSide::Buy,
        order_type: OrderType::LIMIT,
        state: OrderState::Dead,
    };
}

// ── the pool ──────────────────────────────────────────────────────────────────
//
// Memory picture (MAX_CLIENTS=4, MAX_ORDERS_PER_CLIENT=4 for illustration):
//
//   orders[0]: [ Order, Order, Order, Order ]   ← all slots for client 0
//   orders[1]: [ Order, Order, Order, Order ]   ← all slots for client 1
//   orders[2]: [ Order, Order, Order, Order ]
//   orders[3]: [ Order, Order, Order, Order ]
//
// The whole thing is one flat block on the heap: 4*4 = 16 Orders, contiguous.
// Accessing orders[client_id][slot] compiles to:
//   address = base + client_id * (MAX_ORDERS_PER_CLIENT * size_of::<Order>())
//           + slot * size_of::<Order>()
// — one multiply and one add, resolved at compile time.
pub struct OrderPool {
    // Box puts the array on the heap (it's too large for the stack).
    // [[Order; 1024]; 256] means: an array of 256 rows, each row being
    // an array of 1024 Orders.
    orders: Box<[[Order; MAX_ORDERS_PER_CLIENT]; MAX_CLIENTS]>,

    // next_slot[cid] is the slot index we hand out next for client `cid`.
    // It wraps around at MAX_ORDERS_PER_CLIENT (power-of-2 bitmask trick).
    next_slot: [u16; MAX_CLIENTS],
}

impl OrderPool {
    pub fn new() -> Self {
        Self {
            // Allocate the entire grid at once and fill every cell with DEAD.
            // After this line no further heap allocation ever happens for orders.
            orders: Box::new([[Order::DEAD; MAX_ORDERS_PER_CLIENT]; MAX_CLIENTS]),
            next_slot: [0u16; MAX_CLIENTS],
        }
    }

    // ── inserting a new order ─────────────────────────────────────────────────
    //
    // Called when a new order arrives from the client, before we forward it
    // to the engine.
    //
    // Returns the market_order_id to stamp onto the outgoing engine message,
    // or None if the client slot ring is fully occupied (all 1024 slots live).
    pub fn insert(
        &mut self,
        client_id: u32,
        client_order_id: u32,
        market_index: u32,
        price: u128,
        quantity: u128,
        side: OrderSide,
        order_type: OrderType,
    ) -> Option<u32> {
        let cid = client_id as usize;

        // Walk forward from next_slot until we find a Dead slot.
        // In steady state the very first candidate is always Dead (orders
        // complete before the ring wraps), so this is effectively O(1).
        let slot = self.find_free_slot(cid)?;

        // Encode (client_id, slot) into a single u32 that the engine will
        // echo back verbatim. Upper 16 bits = client_id, lower 16 = slot.
        let market_order_id: u32 = ((client_id & 0xFFFF) << 16) | (slot as u32 & 0xFFFF);

        self.orders[cid][slot] = Order {
            client_id,
            client_order_id,
            market_order_id,
            market_index,
            price,
            quantity,
            leaves_qty: quantity,
            side,
            order_type,
            state: OrderState::PendingNew,
        };

        Some(market_order_id)
    }

    // ── looking up by market_order_id (engine response path) ─────────────────
    //
    // The engine echoes market_order_id on every fill/cancel/reject.
    // We decode it directly into (client_id, slot) — zero searching.
    pub fn get_by_market_order_id(&self, market_order_id: u32) -> Option<&Order> {
        let (cid, slot) = Self::decode(market_order_id);
        let order = &self.orders[cid][slot];
        if order.state == OrderState::Dead {
            return None;
        }
        Some(order)
    }

    pub fn get_by_market_order_id_mut(&mut self, market_order_id: u32) -> Option<&mut Order> {
        let (cid, slot) = Self::decode(market_order_id);
        let order = &mut self.orders[cid][slot];
        if order.state == OrderState::Dead {
            return None;
        }
        Some(order)
    }

    // ── state transitions (called from the engine response handler) ───────────

    pub fn on_accepted(&mut self, market_order_id: u32) {
        if let Some(order) = self.get_by_market_order_id_mut(market_order_id) {
            order.state = OrderState::Live;
        }
    }

    // partial_fill=true when leaves_qty > 0, false when fully filled
    pub fn on_fill(&mut self, market_order_id: u32, filled_qty: u128) {
        if let Some(order) = self.get_by_market_order_id_mut(market_order_id) {
            order.leaves_qty = order.leaves_qty.saturating_sub(filled_qty);
            if order.leaves_qty == 0 {
                order.state = OrderState::Dead; // slot is free again
            }
        }
    }

    pub fn on_cancelled(&mut self, market_order_id: u32) {
        if let Some(order) = self.get_by_market_order_id_mut(market_order_id) {
            order.state = OrderState::Dead; // slot is free again
        }
    }

    pub fn mark_pending_cancel(&mut self, market_order_id: u32) {
        if let Some(order) = self.get_by_market_order_id_mut(market_order_id) {
            order.state = OrderState::PendingCancel;
        }
    }

    // ── helpers ───────────────────────────────────────────────────────────────

    fn find_free_slot(&mut self, cid: usize) -> Option<usize> {
        let start = self.next_slot[cid] as usize;
        for i in 0..MAX_ORDERS_PER_CLIENT {
            // power-of-2 wrap: `& (MAX_ORDERS_PER_CLIENT - 1)` instead of `%`
            let slot = (start + i) & (MAX_ORDERS_PER_CLIENT - 1);
            if self.orders[cid][slot].state == OrderState::Dead {
                // advance next_slot past this one for the next call
                self.next_slot[cid] = ((slot + 1) & (MAX_ORDERS_PER_CLIENT - 1)) as u16;
                return Some(slot);
            }
        }
        None // all 1024 slots occupied — backpressure the client
    }

    // Decode the market_order_id we encoded in `insert`.
    #[inline(always)]
    fn decode(market_order_id: u32) -> (usize, usize) {
        let cid = ((market_order_id >> 16) & 0xFFFF) as usize;
        let slot = (market_order_id & 0xFFFF) as usize;
        (cid, slot)
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    fn make_pool() -> OrderPool {
        OrderPool::new()
    }

    #[test]
    fn insert_and_lookup_by_market_order_id() {
        let mut pool = make_pool();

        let mid = pool
            .insert(3, 42, 0, 100, 10, OrderSide::Buy, OrderType::LIMIT)
            .expect("slot should be free");

        // Decode should give us client 3
        let (cid, _slot) = OrderPool::decode(mid);
        assert_eq!(cid, 3);

        let order = pool.get_by_market_order_id(mid).expect("order must exist");
        assert_eq!(order.client_id, 3);
        assert_eq!(order.client_order_id, 42);
        assert_eq!(order.state, OrderState::PendingNew);
    }

    #[test]
    fn accept_then_fill_frees_slot() {
        let mut pool = make_pool();

        let mid = pool
            .insert(1, 99, 0, 200, 5, OrderSide::Sell, OrderType::LIMIT)
            .unwrap();

        pool.on_accepted(mid);
        assert_eq!(
            pool.get_by_market_order_id(mid).unwrap().state,
            OrderState::Live
        );

        pool.on_fill(mid, 5); // fully filled
        assert!(
            pool.get_by_market_order_id(mid).is_none(),
            "slot must be Dead"
        );
    }

    #[test]
    fn slot_is_reused_after_dead() {
        let mut pool = make_pool();

        let mid1 = pool
            .insert(0, 1, 0, 100, 1, OrderSide::Buy, OrderType::LIMIT)
            .unwrap();
        pool.on_accepted(mid1);
        pool.on_fill(mid1, 1); // slot 0 is now Dead

        // next insert for client 0 should reuse slot 0
        let mid2 = pool
            .insert(0, 2, 0, 100, 1, OrderSide::Buy, OrderType::LIMIT)
            .unwrap();
        let (_cid1, slot1) = OrderPool::decode(mid1);
        let (_cid2, slot2) = OrderPool::decode(mid2);
        assert_eq!(slot1, slot2, "slot should be reused");
    }
}
