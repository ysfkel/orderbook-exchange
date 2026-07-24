use common::types::{NewOrder, NewOrderMessage};

#[derive(Clone, Copy, Debug)]
pub enum OrderRequest {
    NewOrder(NewOrder),
}

#[derive(Debug)]
pub enum OutboundMessageType {
    New(NewOrderMessage),
    // Cancel(CancelOrderMessage),   // add when cancel path lands
    // Amend(AmendOrderMessage),     // add when amend path lands
}

impl From<NewOrder> for OutboundMessageType {
    fn from(order: NewOrder) -> Self {
        OutboundMessageType::New(NewOrderMessage::new(order))
    }
}
