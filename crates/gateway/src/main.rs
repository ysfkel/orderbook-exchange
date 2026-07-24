mod error;
mod network;
mod types;
mod service;
mod api;

use std::{sync::{Arc, atomic::AtomicBool}, thread, time::{Duration, SystemTime, UNIX_EPOCH}};
use common::{queue::RingBufferQueue, traits::ThreadHandler, types::NewOrder};
use crossbeam::queue::ArrayQueue;
use crate::{network::outbound::publisher_service::PublisherService, service::OrderService, types::{OrderRequest, OutboundMessageType}};
use common::{queue::{QueueConsumer, QueueProducer, QueueRecvError, QueueSendError, RingBufferConsumer, RingBufferProducer}, thread_handle::ThreadHandle,types::{NewOrderMessage, OrderSide, OrderType, create_order, error::{Disposition, RejectReason}}};
use api::mock_api::MockOrderFeed;

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

    ctrlc::set_handler(move || shutdown_ctlrc.store(true, std::sync::atomic::Ordering::Relaxed))?;

   
    let order_queue: Arc<ArrayQueue<OrderRequest>> = Arc::new(ArrayQueue::new(4096));
  
    let RingBufferQueue {
        producer: outbound_order_producer,
        consumer: outbound_order_consumer,
    }: RingBufferQueue<OutboundMessageType> = RingBufferQueue::new(4096);

    let outbound_handler = PublisherService::new(outbound_order_consumer).start();
    let mock_feed = MockOrderFeed::new(order_queue.clone()).start();
 
    let order_service = OrderService::new(order_queue.clone(), outbound_order_producer).start();

    while !shutdown.load(std::sync::atomic::Ordering::Relaxed) {
        thread::sleep(Duration::from_millis(100));
    }

    outbound_handler.stop();
    order_service.stop();
    mock_feed.stop();

   Ok(())

}

