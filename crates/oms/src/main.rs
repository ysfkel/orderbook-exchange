pub mod error;
mod network;
mod service;
pub mod storage;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use common::queue::RingBufferQueue;
use common::traits::ThreadHandler;
use common::types::{AcceptedOrder, NewOrder};

use crate::network::clients::instruments::InstrumentsClient;
use crate::network::inbound::listener::Listener;
use crate::network::outbound::start::AcceptedOrderPublisher;
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
    let shutdown_ctlrc = shutdown.clone();

    // 2. register ctrlc before spawning threads
    ctrlc::set_handler(move || shutdown_ctlrc.store(true, Ordering::Relaxed))?;

    if let Err(e) = get_instruments().await {
        eprintln!("gRPC call failed: {:?}", e);
    }

    let RingBufferQueue {
        producer: new_order_producer,
        consumer: new_order_consumer,
    }: RingBufferQueue<NewOrder> = RingBufferQueue::new(4096);
    let RingBufferQueue {
        producer: outbound_new_order_producer,
        consumer: outbound_new_order_consumer,
    }: RingBufferQueue<AcceptedOrder> = RingBufferQueue::new(4096);

    let listener = Listener::new(new_order_producer).start();
    let outbound_handler = AcceptedOrderPublisher::new(outbound_new_order_consumer).start();
    let new_order_service =
        NewOrderService::new(new_order_consumer, outbound_new_order_producer).start();
    // Park until Ctrl-C.
    while !shutdown.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_millis(100));
    }

    listener.stop();
    outbound_handler.stop();
    new_order_service.stop();
    tracing::info!("engine stopped");
    Ok(())

    // let t1 = thread::Builder::new()
    //     .name("oms-inbound-orders".into())
    //     .spawn(move || {
    //         start_new_order_listener(shutdown_t1, max_msg_size, new_order_producer);
    //     })?;

    // let t2 = thread::Builder::new()
    //     .name("oms-outbound-orders".into())
    //     .spawn(move || {
    //         start_accepted_order_publisher(outbound_new_order_consumer, shutdown_t2);
    //     })?;

    // // `consumer` is handed to NewOrderService here — it's the other end of
    // // the same ring buffer `producer` was moved into above, on t1. This
    // // thread drains whatever the Aeron listener pushes.
    // let t3 = thread::Builder::new()
    //     .name("oms-new-order-service".into())
    //     .spawn(move || {
    //         let mut new_order_service =
    //             NewOrderService::new(new_order_consumer, outbound_new_order_producer);
    //         new_order_service.run(shutdown_t3);
    //     })?;

    // t1.join().expect("inbound-orders thread panicked");
    // t2.join().expect("inbound-reports thread panicked");
}

async fn get_instruments() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = InstrumentsClient::connect("http://127.0.0.1:50051").await?;
    let markets = client.get_instruments().await?;

    for market in markets {
        println!("MARKET={:?}", market);
    }

    Ok(())
}
