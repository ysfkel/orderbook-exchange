For a low-latency exchange system, eventually you will likely want:

busy-spin instead of sleeping
SPSC ring buffer between Aeron and matching engine
pinned threads
batching fragments
zero-copy parsing
fixed-size binary messages instead of UTF-8 strings

----

continue at crates/engine/orderbook.rs 
implement fn add()

continue at crates/engine/orderbook.rs 
implement 
pub fn cancel(&mut self, client_id: ClientId, order_id: OrderId, ticker_id: TickerId) {

implemment matching engine at 
rates/engine/matching_engine.rs