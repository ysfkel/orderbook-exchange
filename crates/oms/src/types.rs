use common::types::{AcceptedOrder, CreateOrderMessage};

#[derive(Debug)]
pub enum OutboundMessageType {
    New(CreateOrderMessage),
    // Cancel(CancelOrderMessage),   // add when cancel path lands
    // Amend(AmendOrderMessage),     // add when amend path lands
}

impl From<AcceptedOrder> for OutboundMessageType {
    fn from(order: AcceptedOrder) -> Self {
        OutboundMessageType::New(CreateOrderMessage::new(order))
    }
}
