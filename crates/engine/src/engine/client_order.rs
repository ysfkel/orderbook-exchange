// use common::{
//     mem_pool::POOL_IDX_NULL,
//     types::{ME_MAX_NUM_CLIENTS, ME_MAX_ORDER_IDS, PoolIdx},
// };

// pub type OrderPoolIdx = Vec<PoolIdx>;
// pub type ClientOrderPoolIdx = Vec<OrderPoolIdx>;

// pub struct ClientOrder {
//     cid_oid_to_order: ClientOrderPoolIdx,
// }

// impl ClientOrder {
//     pub fn new() -> Self {
//         Self {
//             cid_oid_to_order: vec![vec![POOL_IDX_NULL; ME_MAX_ORDER_IDS]; ME_MAX_NUM_CLIENTS],
//         }
//     }
// }
