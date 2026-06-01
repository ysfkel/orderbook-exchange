
use transport::AeronTransport;
use transport::{Publisher, Subscriber};
use std::thread;
use std::time::Duration;

use crate::error::ProgramError;

const GATEWAY_NEW_ORDER: &str = "aeron:udp?endpoint=127.0.0.1:40455";
const AERON_DIR: &str = "/tmp/aeron-exchange";
const ORDER_STREAM_ID: i32 = 1000;

pub fn start() {

        let tp = AeronTransport::connect(&AERON_DIR).expect("error connecting to aeron");
       let p = tp.subscriber(GATEWAY_NEW_ORDER, ORDER_STREAM_ID, |msg: &[u8]| {
        match std::str::from_utf8(msg) {
            Ok(s) => println!("message received: {:?}", s),
            Err(e) => println!("failed to parse message: {:?}", e),
        }

    } ).expect("failed to get publisher");
    
    let mut tx = OrderSubscriber::new(p).expect("failed to create publisher");
    match tx.subscribe() {
        Ok(r) => println!("subscription ended: {:?}", r),
        Err(e) => eprintln!("subscription error: {:?}", e),
    }
}


pub struct OrderSubscriber<T: Subscriber> {
    transport: T,
}

 /// listens to new order messages from gateway 
impl<T: Subscriber> OrderSubscriber<T> where ProgramError: From<T::Error>{

    pub fn new(transport: T) -> Result<Self, ProgramError> {
 
          Ok(Self {
            transport
          })
    }

    pub fn subscribe(&mut self,) -> Result<(), ProgramError>  {
        loop {
             match self.transport.poll(10) {
                Ok(fragment) => {
                  //  println!("received fragment: {}", fragment)
                },
                Err(e) => eprintln!("poll error: {:?}", e),
             }
            thread::sleep(Duration::from_millis(1));

        }
      
    }
}
