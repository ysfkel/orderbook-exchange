use common::{
    mem_pool::{MemPool, POOL_IDX_NULL},
    types::{
        ClientId, ME_MAX_NUM_CLIENTS, ME_MAX_ORDER_IDS, ME_MAX_PRICE_LEVELS, OrderId, OrderSide,
        PoolIdx, Price, Priority, Qty, TickerId,
    },
};

use crate::engine::{
    order::{self, Order},
    order_at_price::{OrdersAtPrice, PriceOrderAtPrice},
    types::ClientOrderPoolIdx,
};

pub struct OrderBook {
    pub ticker_id: TickerId,
    // cid_oid_to_order
    cid_oid_to_order: ClientOrderPoolIdx,

    /// A box that holds every order at one price.
    /// eg That's it. If three people all want to sell at price 103, their three orders go in the same box — the price-103 box
    /// The box itself only remembers head_order (first_me_order) (the first order in line); each order then points to the next one behind it. That vertical chain is the queue — first come, first served.
    orders_at_price_pool: MemPool<OrdersAtPrice>,
    /// Best bid level (head of the circular list of bid levels), or NULL.
    best_bid_price_level: PoolIdx,
    /// Best ask level, or NULL.
    best_ask_price_level: PoolIdx,
    /// price_to_index(price) -> price-level handle.

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
    price_to_level_idx: Vec<PoolIdx>,
    order_pool: MemPool<Order>,
    next_market_order_id: OrderId,
}

impl OrderBook {
    pub fn new(ticker_id: TickerId) -> Self {
        Self {
            ticker_id,
            cid_oid_to_order: vec![vec![POOL_IDX_NULL; ME_MAX_ORDER_IDS]; ME_MAX_NUM_CLIENTS],
            orders_at_price_pool: MemPool::new(ME_MAX_PRICE_LEVELS),
            best_bid_price_level: POOL_IDX_NULL,
            best_ask_price_level: POOL_IDX_NULL,
            price_to_level_idx: vec![POOL_IDX_NULL; ME_MAX_PRICE_LEVELS],
            order_pool: MemPool::new(ME_MAX_ORDER_IDS),
            next_market_order_id: 1,
        }
    }

    /// This is the top-level entry point called when a new order arrives at the matching engine.
    /// It handles the full lifecycle of a new order in one shot: acknowledge it, try to match it
    ///  immediately, and if anything is left over, rest it in the book.
    ///
    /// All the parameters are what the gateway sends in: who submitted it, their own order ID, which instrument,
    /// buy or sell, at what price, and how many units.
    pub fn add(
        &mut self,
        client_id: ClientId,
        client_order_id: OrderId,
        ticker_id: TickerId,
        side: OrderSide,
        price: Price,
        quantity: Qty,
    ) {
        // The engine assigns its own internal ID — independent of client_order_id. The client may reuse their own IDs across reconnects; the engine's ID is unique for the lifetime of the book.
        let new_market_order_id = self.generate_new_market_order_id();

        // todo!
        // ACCEPTED (send accept message to the client-participant who sent the order) goes out immediately — minimizing time-to-ack matters for
        // latency-sensitive participants, even before matching runs.

        let leaves_qty = self.check_for_match(
            client_id,
            client_order_id,
            ticker_id,
            side,
            price,
            quantity,
            new_market_order_id,
        );

        // if it is a partial fill
        // park the remaining in the order book
        if leaves_qty > 0 {
            let priotity = self.get_next_priority(price);
            // get empty pool index
            let empty_order_pool_index = self.order_pool.allocate();
            {
                // get free pool object
                let o = self.order_pool.get_mut(empty_order_pool_index);
                o.ticker_id = ticker_id;
                o.client_id = client_id;
                o.client_order_id = client_order_id;
                o.market_order_id = new_market_order_id;
                o.side = side;
                o.price = price;
                o.quantity = leaves_qty;
                o.priority = priotity;
            }
            self.add_order(empty_order_pool_index);

            // todo! send send_market_update -> see the original converted rust code
            // TODO!! self.send_market_update.
            //
            //
        }
    }

    // ------------------------------------------------------------------
    // Public API used by the MatchingEngine: add() and cancel().
    // ------------------------------------------------------------------

    pub fn cancel(&mut self, client_id: ClientId, order_id: OrderId, ticker_id: TickerId) {
        let valid_ids = (client_id as usize) < self.cid_oid_to_order.len()
            && (order_id as usize) < ME_MAX_ORDER_IDS;

        // retreive order index
        let order_index = if valid_ids {
            self.cid_oid_to_order[client_id as usize][order_id as usize]
        } else {
            POOL_IDX_NULL
        };

        if order_index == POOL_IDX_NULL {
            // TODO!! send_client_response
        }

        // TODO! send_client_response
        // TODO! send_market_update

        self.remove_order(order_index);
    }

    fn add_order(&mut self, order_pool_index: PoolIdx) {
        let (client_id, client_order_id, side, price) = {
            let o = self.order_pool.get(order_pool_index);
            (o.client_id, o.client_order_id, o.side, o.price)
        };

        let level = self.get_price_level_index(price);

        // if the price level index (in the pool) is empty
        if level == POOL_IDX_NULL {
            // No level at this price yet: order is alone in its FIFO.
            {
                // this is a circular doubly-linked list, and the single-element case is
                // just the degenerate form of that invariant — not a special case.
                // The invariant for a circular list is: every node's next eventually
                //  wraps back to it, and every node's prev does too. For a one-element list,
                // "the next node" and "the previous node" are both the node itself. So:
                let o = self.order_pool.get_mut(order_pool_index);
                o.prev_order = order_pool_index;
                o.next_order = order_pool_index;
            }

            let new_level_index = self.orders_at_price_pool.allocate();
            {
                let l = self.orders_at_price_pool.get_mut(new_level_index);
                l.side = side;
                l.price = price;
                l.head_order = order_pool_index;
            }
            self.insert_price_level(new_level_index);
        } else {
            // Append at the tail of the FIFO (i.e., insert before `first`).
            let first = self.orders_at_price_pool.get(level).head_order;
            let tail = self.order_pool.get(first).prev_order;
            {
                let o = self.order_pool.get_mut(order_pool_index);
                o.prev_order = tail;
                o.next_order = first;
            }
            self.order_pool.get_mut(tail).next_order = order_pool_index;
            self.order_pool.get_mut(first).prev_order = order_pool_index;
        }
    }
    // ------------------------------------------------------------------
    // Price-level list maintenance (addOrdersAtPrice / removeOrdersAtPrice).
    // ------------------------------------------------------------------

    /// Inserts an already-allocated price level into this side's price-ordered
    /// doubly-linked list (`bids_by_price` / `asks_by_price`), and registers it
    /// in `price_to_index` for O(1) lookup by price.
    ///
    /// The level itself (struct, side, price, first_me_order) must already be
    /// populated by the caller — this function only handles *linking*: walking
    /// the existing chain to find the correct sorted position by price and
    /// rewiring neighbor pointers (or starting a new circular list if this is
    /// the first level on this side).
    fn insert_price_level(&mut self, new_level: PoolIdx) {
        let (side, new_price) = {
            let l = self.orders_at_price_pool.get(new_level);
            (l.side, l.price)
        };

        self.price_to_level_idx[Self::price_to_index(new_price)] = new_level;

        let head = match side {
            OrderSide::Buy => self.best_bid_price_level,
            OrderSide::Sell => self.best_ask_price_level,
            OrderSide::Unset => unreachable!("invalid side on price level"),
        };

        if head == POOL_IDX_NULL {
            // First level on this side: a one-element circular list.
            let l = self.orders_at_price_pool.get_mut(new_level);
            l.prev_order = new_level;
            l.next_order = new_level;
            match side {
                OrderSide::Buy => self.best_bid_price_level = new_level,
                _ => self.best_ask_price_level = new_level,
            }
            return;
        }

        // Walk from the best level to find the first level the new one should
        // sit *before* (bids: descending, asks: ascending). If we wrap around,
        // the new level is the worst price and goes at the tail.
        let mut cur = head;
        let mut new_is_head = false;
        loop {
            let cur_price = self.orders_at_price_pool.get(cur).price;
            let goes_before = match side {
                OrderSide::Buy => new_price > cur_price,
                _ => new_price < cur_price,
            };
            if goes_before {
                new_is_head = cur == head;
                break;
            }
            cur = self.orders_at_price_pool.get(cur).next_order;
            if cur == head {
                break; // wrapped: insert before head == append at tail
            }
        }
        // Insert new_level before `cur` in the circular list.
        let prev = self.orders_at_price_pool.get(new_level).prev_order;
        {
            let l = self.orders_at_price_pool.get_mut(new_level);
            l.prev_order = prev;
            l.next_order = cur;
        }
        self.orders_at_price_pool.get_mut(prev).next_order = new_level;
        self.orders_at_price_pool.get_mut(cur).prev_order = new_level;

        if new_is_head {
            match side {
                OrderSide::Buy => self.best_bid_price_level = new_level,
                _ => self.best_ask_price_level = new_level,
            }
        }
    }

    // ------------------------------------------------------------------
    // Small private helpers (generateNewMarketOrderId, priceToIndex,
    // getOrdersAtPrice from the book).
    // ------------------------------------------------------------------
    fn generate_new_market_order_id(&mut self) -> OrderId {
        let id = self.next_market_order_id;
        self.next_market_order_id += 1;
        id
    }

    /// Check the new order against the opposite side of the book and execute
    /// any crossing quantity. Returns leftover qty (0 if fully filled,
    /// original qty if nothing crossed).
    fn check_for_match(
        &mut self,
        client_id: ClientId,
        client_order_id: OrderId,
        ticker_id: TickerId,
        side: OrderSide,
        price: Price,
        quantity: Qty,
        new_market_order_id: OrderId,
    ) -> Qty {
        let mut leaves_qty = quantity;
        match side {
            OrderSide::Buy => {
                leaves_qty = self.match_against_side(
                    client_id,
                    client_order_id,
                    ticker_id,
                    side,
                    price,
                    quantity,
                    self.best_ask_price_level,
                    new_market_order_id,
                );
            }
            OrderSide::Sell => {
                leaves_qty = self.match_against_side(
                    client_id,
                    client_order_id,
                    ticker_id,
                    side,
                    price,
                    quantity,
                    self.best_bid_price_level,
                    new_market_order_id,
                );
            }
            OrderSide::Unset => {}
        }

        leaves_qty
    }

    #[inline]
    fn match_against_side(
        &mut self,

        client_id: ClientId,
        client_order_id: OrderId,
        ticker_id: TickerId,
        side: OrderSide,
        price: Price,
        quantity: Qty,
        best_price_level_for_side: PoolIdx, // side eg buy / sell
        new_market_order_id: OrderId,
    ) -> Qty {
        let mut leaves_qty = quantity;
        while leaves_qty > 0 && best_price_level_for_side != POOL_IDX_NULL {
            let best = best_price_level_for_side;
            let (best_price, first) = {
                let l = self.orders_at_price_pool.get(best);
                (l.price, l.head_order)
            };

            if price < best_price {
                break; // no longer crossing
            }
            leaves_qty = self.match_order(
                ticker_id,
                client_id,
                side,
                client_order_id,
                new_market_order_id,
                first,
                leaves_qty,
            );
        }
        leaves_qty
    }

    // ------------------------------------------------------------------
    // Matching (the book's match() and checkForMatch()).
    // ------------------------------------------------------------------

    /// Execute the aggressor against one resting order (`itr`). Returns the
    /// aggressor's remaining quantity.
    fn match_order(
        &mut self,
        ticker_id: TickerId,
        client_id: ClientId,
        side: OrderSide,
        client_order_id: OrderId,
        new_market_order_id: OrderId,
        itr: PoolIdx,
        mut leaves_qty: Qty,
    ) -> Qty {
        // get resting order
        let mut passive = *self.order_pool.get_mut(itr);
        let fill_qty = leaves_qty.min(passive.quantity);
        leaves_qty -= fill_qty;
        passive.quantity -= fill_qty;

        // 1. todo! Send Fill report for the aggressive order - see converted rust code

        // 2. todo! send Fill repost for  the passive order it traded against.

        // 3. Anonymous TRADE message for the public feed.

        if passive.quantity == 0 {
            // The order is fully consumed by trades -> Order is no longer resting in the book
            // todo! send_market_update -> see original rust converted code

            // remove the consumed passive order from the book
            self.remove_order(itr)
        } else {
            //  todo! send_market_update -> see original rust converted code
        }

        leaves_qty
    }

    fn remove_order(&mut self, order_idx: PoolIdx) {
        let (client_id, client_order_id, side, price, prev_order, next_order) = {
            let order = self.order_pool.get(order_idx);
            (
                order.client_id,
                order.client_order_id,
                order.side,
                order.price,
                order.prev_order,
                order.next_order,
            )
        };

        if next_order == order_idx {
            // only order at this price: drop the whole level
            self.remove_orders_at_price(side, price);
        } else {
            //  order being removed is NOT the only order at that price level.
            // we are removing a node from a circular doubly-linked list of orders at a given price level.

            // Step 1: bridge the previous node to the next node (forward direction)
            // prev -> order_idx -> next  becomes  prev -> next
            self.order_pool.get_mut(prev_order).next_order = next_order;

            // Step 2: bridge the next node back to the previous node (backward direction)
            // This restores the doubly-linked structure after removing order_idx
            self.order_pool.get_mut(next_order).prev_order = prev_order;

            // Step 3: fetch the price level structure that contains metadata
            // including the "first_me_order" pointer (head of FIFO at this price)
            let level = self.get_price_level_index(price);

            let l = self.orders_at_price_pool.get_mut(level);

            if l.head_order == order_idx {
                l.head_order = next_order;
            }

            self.cid_oid_to_order[client_id as usize][client_order_id as usize] = POOL_IDX_NULL;
            self.order_pool.deallocate(order_idx);
        }
    }

    fn remove_orders_at_price(&mut self, side: OrderSide, price: Price) {
        let level = self.get_price_level_index(price);

        debug_assert!(level != POOL_IDX_NULL);

        let (prev, next) = {
            let l = self.orders_at_price_pool.get(level);
            (l.prev_order, l.next_order)
        };

        if next == level {
            // Only level on this side.
            match side {
                OrderSide::Buy => self.best_bid_price_level = POOL_IDX_NULL,
                _ => self.best_ask_price_level = POOL_IDX_NULL,
            }
        } else {
            // This block is handling the case where you are removing a price level node
            // from a circular doubly-linked list of price levels, but it is NOT the only
            //  price level on that side (bids/asks).
            self.orders_at_price_pool.get_mut(prev).next_order = next;
            self.orders_at_price_pool.get_mut(next).prev_order = prev;

            if self.best_bid_price_level == level {
                self.best_bid_price_level = next;
            }

            if self.best_ask_price_level == level {
                self.best_ask_price_level = next;
            }
        }

        // invalidate price → level mapping (level was deallocated)
        self.price_to_level_idx[Self::price_to_index(price)] = POOL_IDX_NULL;
        self.orders_at_price_pool.deallocate(level);
    }

    fn get_price_level_index(&self, price: Price) -> PoolIdx {
        self.price_to_level_idx[Self::price_to_index(price)]
    }

    fn price_to_index(price: Price) -> usize {
        price.rem_euclid(ME_MAX_PRICE_LEVELS as Price) as usize
    }

    /// Priority for a new order at `price`: 1 if the level doesn't exist yet,
    /// otherwise last order's priority + 1 (FIFO position).
    fn get_next_priority(&self, price: Price) -> Priority {
        let level = self.get_price_level_index(price);

        if level == POOL_IDX_NULL {
            return 1u64;
        }

        let first = self.orders_at_price_pool.get(level).head_order;
        // Circular list: the tail is first->prev.
        let tail = self.order_pool.get(first).prev_order;
        self.order_pool.get(tail).priority + 1
    }
}
