use std::thread;

use transport::AeronTransport;
use super::accepted_order::OrderBookPublisher;

const ORDER_CHANNEL: &str = "aeron:udp?endpoint=127.0.0.1:40456";
const AERON_DIR: &str = "/tmp/aeron-exchange";
const ORDER_STREAM_ID: i32 = 1001;

pub fn run() {

        let tp = AeronTransport::connect(&AERON_DIR).expect("error connecting to aeron");
        let p = tp.publisher(ORDER_CHANNEL, ORDER_STREAM_ID).expect("failed to get publisher");
        let publisher = OrderBookPublisher::new(p).expect("failed to create publisher");

      loop {{
          let _ = publisher.publish("hello aeron".as_bytes());
          thread::sleep(std::time::Duration::from_secs(1));
      }}
}