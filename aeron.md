


```
use rusteron_client::*;
use std::time::Duration;

const ORDER_CHANNEL: &str = "aeron:udp?endpoint=127.0.0.1:40456";
const ORDER_STREAM_ID: i32 = 1001;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = AeronContext::new()?;
    ctx.set_dir(&"/tmp/aeron-exchange".into_c_string())?;  // MUST match the driver

    let aeron = Aeron::new(&ctx)?;
    aeron.start()?;

    let publication = aeron
        .async_add_publication(ORDER_CHANNEL, ORDER_STREAM_ID)?
        .poll_blocking(Duration::from_secs(5))?;

    // build your CreateOrderMessage as before
    let msg = build_create_order_message();
    let bytes = msg.as_bytes();

    // offer returns the new position, or a negative value on backpressure
    loop {
        let result = publication.offer(
            bytes,
            Handlers::no_reserved_value_supplier_handler(),
        );
        if result >= 0 { break; }
        // BACK_PRESSURED (-2), NOT_CONNECTED (-1), ADMIN_ACTION (-3), CLOSED (-4)
        std::thread::yield_now();
    }
    Ok(())
}

```

engine 

```
use rusteron_client::*;
use std::cell::Cell;
use std::time::Duration;

const ORDER_CHANNEL: &str = "aeron:udp?endpoint=127.0.0.1:40456";
const ORDER_STREAM_ID: i32 = 1001;

struct OrderHandler;

impl AeronFragmentHandlerCallback for OrderHandler {
    fn handle_aeron_fragment_handler(&mut self, buffer: &[u8], _header: AeronHeader) {
        // same dispatch logic you have today in handle_message
        if let Err(e) = crate::network::handle_message(buffer, 512) {
            tracing::warn!(error = %e, "handle_message failed");
        }
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = AeronContext::new()?;
    ctx.set_dir(&"/tmp/aeron-exchange".into_c_string())?;

    let aeron = Aeron::new(&ctx)?;
    aeron.start()?;

    let subscription = aeron
        .async_add_subscription(
            ORDER_CHANNEL,
            ORDER_STREAM_ID,
            Handlers::no_available_image_handler(),
            Handlers::no_unavailable_image_handler(),
        )?
        .poll_blocking(Duration::from_secs(5))?;

    // fragment assembler stitches packets > MTU back together
    let (handler, _inner) =
        Handler::leak_with_fragment_assembler(OrderHandler)?;

    loop {
        let _ = subscription.poll(Some(&handler), 1024)?;  // 1024 = fragment limit
    }
}

```