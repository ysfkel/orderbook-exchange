// use std::thread;
// use std::time::Duration;
// use transport::AeronTransport;
// use transport::{Publisher, Subscriber};

// use crate::error::ProgramError;

// const ORDER_CHANNEL: &str = "aeron:udp?endpoint=127.0.0.1:40456";
// const AERON_DIR: &str = "/tmp/aeron-exchange";
// const ORDER_STREAM_ID: i32 = 1001;

// pub fn run() {
//     let tp = AeronTransport::connect(&AERON_DIR).expect("error connecting to aeron");
//     let p = tp
//         .subscriber(
//             ORDER_CHANNEL,
//             ORDER_STREAM_ID,
//             |msg: &[u8]| match std::str::from_utf8(msg) {
//                 Ok(s) => println!("message received: {:?}", s),
//                 Err(e) => println!("failed to parse message: {:?}", e),
//             },
//         )
//         .expect("failed to get publisher");

//     let mut tx = OrderSubscriber::new(p).expect("failed to create publisher");
//     match tx.subscribe() {
//         Ok(r) => println!("subscription ended: {:?}", r),
//         Err(e) => eprintln!("subscription error: {:?}", e),
//     }
// }

use std::thread;
use std::time::Duration;
use transport::AeronTransport;
use transport::{Publisher, Subscriber};

use crate::error::ProgramError;

const ORDER_CHANNEL: &str = "aeron:udp?endpoint=127.0.0.1:40456";
const AERON_DIR: &str = "/tmp/aeron-exchange";
const ORDER_STREAM_ID: i32 = 1001;

pub fn run() {
    let tp = AeronTransport::connect(&AERON_DIR).expect("error connecting to aeron");

    // Same split as new_order_listener.rs: build the message-handling
    // closure once (this is where anything you capture — here, nothing
    // beyond the printlns — would move in), then create the subscription
    // as a separate step that just borrows it. `fragment_handler` must
    // stay alive for as long as `p` (and anything built from it) is used,
    // which is why it's a local binding here rather than a temporary.
    let fragment_handler = AeronTransport::build_fragment_handler(|msg: &[u8]| {
        match std::str::from_utf8(msg) {
            Ok(s) => println!("message received: {:?}", s),
            Err(e) => println!("failed to parse message: {:?}", e),
        }
    })
    .expect("failed to build fragment handler");

    let p = tp
        .add_subscription(ORDER_CHANNEL, ORDER_STREAM_ID, &fragment_handler)
        .expect("failed to get subscriber");

    let mut tx = OrderSubscriber::new(p).expect("failed to create publisher");
    match tx.subscribe() {
        Ok(r) => println!("subscription ended: {:?}", r),
        Err(e) => eprintln!("subscription error: {:?}", e),
    }
}

pub struct OrderSubscriber<T: Subscriber> {
    transport: T,
}

impl<T: Subscriber> OrderSubscriber<T>
where
    ProgramError: From<T::Error>,
{
    pub fn new(transport: T) -> Result<Self, ProgramError> {
        Ok(Self { transport })
    }

    pub fn subscribe(&mut self) -> Result<(), ProgramError> {
        loop {
            match self.transport.poll(10) {
                Ok(fragment) => {
                    //  println!("received fragment: {}", fragment)
                }
                Err(e) => eprintln!("poll error: {:?}", e),
            }
            thread::sleep(Duration::from_millis(1));
        }
    }
}
