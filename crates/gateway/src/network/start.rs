use super::new_order::OrderPublisher;
use crate::network::config::{AERON_DIR, NEW_ORDER_CHANNEL, NEW_ORDER_STREAM_ID};
use common::types::{NewOrder, NewOrderMessage, OrderDTO, OrderSide, OrderType, create_order};
use std::thread;
use transport::AeronTransport;
//
use zerocopy::IntoBytes;

pub fn run() {
    let tp = AeronTransport::connect(&AERON_DIR).expect("error connecting to aeron");
    let p = tp
        .publisher(NEW_ORDER_CHANNEL, NEW_ORDER_STREAM_ID)
        .expect("failed to get publisher");
    let publisher = OrderPublisher::new(p).expect("failed to create publisher");

    let new_order = NewOrder::new(23600, 18, 1000, 1, 1, OrderSide::Buy, OrderType::MARKET);

    let order_message = NewOrderMessage::new(new_order);

    let create_order_bytes = order_message.as_bytes();

    loop {
        {
            let _ = publisher.publish(create_order_bytes);
            thread::sleep(std::time::Duration::from_secs(1));
        }
    }
}
