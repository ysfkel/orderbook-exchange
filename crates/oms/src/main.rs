pub mod error;
mod network;
mod service;
pub mod storage;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use common::queue::RingBufferQueue;
use common::types::NewOrder;

use crate::network::clients::instruments::InstrumentsClient;
use crate::network::inbound::new_order_listener::start_new_order_listener;
use crate::network::outbound::start::run;
use crate::service::new_order::NewOrderService;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_file(false)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_t1 = shutdown.clone();
    let shutdown_t2 = shutdown.clone();
    let shutdown_t3 = shutdown.clone();
    let shutdown_ctlrc = shutdown.clone();

    // 2. register ctrlc before spawning threads
    ctrlc::set_handler(move || shutdown_ctlrc.store(true, Ordering::Relaxed))?;

    if let Err(e) = get_instruments().await {
        eprintln!("gRPC call failed: {:?}", e);
    }
    // todo
    //1 pass order to ringbudder which  pushes to aeron transport publisher
    println!("starting publisher");
    let max_msg_size = 512;

    let RingBufferQueue{ producer, consumer }: RingBufferQueue<NewOrder> = RingBufferQueue::new(4096);

    let t1 = thread::Builder::new()
        .name("oms-inbound-orders".into())
        .spawn(move || {
            start_new_order_listener(shutdown_t1, max_msg_size, producer);
        })?;

    let t2 = thread::Builder::new()
        .name("oms-outbound-orders".into())
        .spawn(move || {
            run(shutdown_t2);
        })?;

      // `consumer` is handed to NewOrderService here — it's the other end of
    // the same ring buffer `producer` was moved into above, on t1. This
    // thread drains whatever the Aeron listener pushes.
    let t3 = thread::Builder::new()
        .name("oms-new-order-service".into())
        .spawn(move || {
            let mut new_order_service = NewOrderService::new(consumer);
            new_order_service.run(shutdown_t3);
        })?;

    t1.join().expect("inbound-orders thread panicked");
    t2.join().expect("inbound-reports thread panicked");

    Ok(())
}

async fn get_instruments() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = InstrumentsClient::connect("http://127.0.0.1:50051").await?;
    let markets = client.get_instruments().await?;

    for market in markets {
        println!("MARKET={:?}", market);
    }

    Ok(())
}
