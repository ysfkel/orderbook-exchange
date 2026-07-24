use common::types::PoolIdx;


pub type OrderPoolIdx = Vec<PoolIdx>;
pub type ClientOrderPoolIdx = Vec<OrderPoolIdx>; // client_id -> order_id -> pool_index
