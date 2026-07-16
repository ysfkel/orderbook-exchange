use common::types::AcceptedOrder;

#[derive(Clone, Copy, Debug)]
pub enum EngineRequest {
    NewOrder(AcceptedOrder),
}
