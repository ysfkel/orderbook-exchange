use super::accepted_order::OrderPublisher;
use crate::network::config::{ACCEPTED_ORDER_CHANNEL, ACCEPTED_ORDER_STREAM_ID, AERON_DIR};
use std::{
    sync::{Arc, atomic::AtomicBool},
    thread,
};
use transport::AeronTransport;

pub fn run(shutdown: Arc<AtomicBool>) {
    let tp = AeronTransport::connect(&AERON_DIR).expect("error connecting to aeron");
    let p = tp
        .publisher(ACCEPTED_ORDER_CHANNEL, ACCEPTED_ORDER_STREAM_ID)
        .expect("failed to get publisher");
    let publisher = OrderPublisher::new(p).expect("failed to create publisher");

    loop {
        {
            let _ = publisher.publish("hello aeron".as_bytes());
            thread::sleep(std::time::Duration::from_secs(1));
        }
    }
}
