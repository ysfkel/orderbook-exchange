use crate::error::{PollError, PublishError, SetupError};
use crate::traits::{Publisher, Subscriber};
use rusteron_client::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tracing::error;

const CONNECTION_TIMEOUT: Duration = Duration::from_secs(5);

/// One per process. Holds the Aeron client connection to the local `aeronmd`.
/// Build publishers and subscribers from it.
pub struct AeronTransport {
    aeron: Aeron,
}

struct FnAdapter<F>(F);

impl<F: FnMut(&[u8])> AeronFragmentHandlerCallback for FnAdapter<F> {
    fn handle_aeron_fragment_handler(&mut self, msg: &[u8], _h: AeronHeader) {
        (self.0)(msg)
    }
}

struct DisconnectHandler(Arc<AtomicBool>);

impl AeronUnavailableImageCallback for DisconnectHandler {
    fn handle_aeron_on_unavailable_image(
        &mut self,
        _subscription: AeronSubscription,
        _image: AeronImage,
    ) {
        self.0.store(true, Ordering::Relaxed);
    }
}

unsafe impl Send for DisconnectHandler {}
unsafe impl Sync for DisconnectHandler {}

impl AeronTransport {
    /// Connect to the running `aeronmd` whose CnC file lives at `aeron_dir`.
    /// Times out if the driver isn't running.  
    pub fn connect(aeron_path: &str) -> Result<Self, SetupError> {
        let ctx = AeronContext::new()?;
        ctx.set_dir(&aeron_path.into_c_string())?;
        ctx.set_driver_timeout_ms(5_000)?;

        // Redirect C-level aeron errors through tracing instead of raw stderr.
        // Without this, aeronmd shutdown prints "(1000): MediaDriver has been
        // shutdown" directly to stderr, bypassing all Rust error handling.
        ctx.set_error_handler(Some(&Handler::leak(AeronErrorHandlerLogger)))?;

        let aeron = Aeron::new(&ctx)?;
        aeron.start()?;
        Ok(Self { aeron })
    }

    pub fn publisher(&self, channel: &str, stream_id: i32) -> Result<AeronPublisher, SetupError> {
        let publication = self
            .aeron
            .async_add_publication(&channel.into_c_string(), stream_id)?
            .poll_blocking(CONNECTION_TIMEOUT)?;
        Ok(AeronPublisher { publication })
    }

    pub fn subscriber<F>(
        &self,
        channel: &str,
        stream_id: i32,
        handler: F,
    ) -> Result<AeronSubscriber<F>, SetupError>
    where
        F: FnMut(&[u8]) + Send + 'static,
    {
        let disconnected = Arc::new(AtomicBool::new(false));
        let unavailable_handler = Handler::leak(DisconnectHandler(disconnected.clone()));

        let subscription: AeronSubscription = self
            .aeron
            .async_add_subscription(
                &channel.into_c_string(),
                stream_id,
                Handlers::no_available_image_handler(),
                Some(&unavailable_handler),
            )?
            .poll_blocking(CONNECTION_TIMEOUT)?;

        // let subscription: AeronSubscription = self
        //     .aeron
        //     .async_add_subscription(
        //         &channel.into_c_string(),
        //         stream_id,
        //         Handlers::no_available_image_handler(),
        //         Handlers::no_unavailable_image_handler(),
        //     )?
        //     .poll_blocking(CONNECTION_TIMEOUT)?;

        let (closure, _inner) = Handler::leak_with_fragment_assembler(FnAdapter(handler))?;

        Ok(AeronSubscriber {
            subscription,
            closure,
            _inner,
            disconnected,
        })
    }
}

// ─── Publisher ──────────────────────────────────────────────────────────────

pub struct AeronPublisher {
    publication: AeronPublication,
}

impl Publisher for AeronPublisher {
    type Data = i64;
    type Error = PublishError;

    #[inline]
    fn publish(&self, bytes: &[u8]) -> Result<Self::Data, Self::Error> {
        let r = self
            .publication
            .offer(bytes, Handlers::no_reserved_value_supplier_handler());

        if r.is_negative() {
            Err(r.into())
        } else {
            Ok(r)
        }
    }
}

// ---- Subscriber ─────────────────────────────────────────────────────────────
pub struct AeronSubscriber<F> {
    subscription: AeronSubscription,
    closure: Handler<AeronFragmentAssembler>, // exact type per your rusteron version
    _inner: Handler<FnAdapter<F>>,            // ditto — kept alive to keep `closure` valid
    disconnected: Arc<AtomicBool>,            // ← was missing from struct
}

impl<F: Send + 'static> Subscriber for AeronSubscriber<F> {
    type Data = usize;
    type Error = PollError;

    #[inline]
    fn poll(&mut self, fragment_limit: usize) -> Result<Self::Data, Self::Error> {
        let r = self
            .subscription
            .poll(Some(&self.closure), fragment_limit)? as usize;
        Ok(r)
    }

    #[inline]
    fn is_connected(&self) -> bool {
        !self.disconnected.load(Ordering::Relaxed)
    }
}

struct AeronErrorHandlerLogger;

impl AeronErrorHandlerCallback for AeronErrorHandlerLogger {
    fn handle_aeron_error_handler(&mut self, code: i32, msg: &str) {
        tracing::error!(code, message = %msg, "aeron driver error");
    }
}
