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

/// Owns the leaked, process-lifetime message-handling closure.
///
/// Build this ONCE, outside any reconnect loop — whatever state your
/// closure captures (e.g. a queue producer) gets moved in here a single
/// time, for good. `add_subscription` below only ever *borrows* this, so
/// you can reconnect as many times as you like without needing to move
/// (or Arc/Mutex-wrap) the captured state again.
pub struct FragmentHandler<F> {
    // The fragment-assembler-wrapped handler actually registered with Aeron
    // on `subscription.poll(...)`.
    closure: Handler<AeronFragmentAssembler>,
    // Kept alive only to keep `closure`'s internal pointer valid — never
    // used directly, hence the underscore.
    _inner: Handler<FnAdapter<F>>,
}

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

    /// Build the message-handling closure once. Call this a single time,
    /// outside your reconnect loop, and hang on to the returned
    /// `FragmentHandler` for as long as you want to keep reconnecting.
    ///
    /// `handler` is whatever `FnMut(&[u8])` you want run per-message — this
    /// is where things like a queue producer get moved in, exactly once.
    pub fn build_fragment_handler<F>(handler: F) -> Result<FragmentHandler<F>, SetupError>
    where
        F: FnMut(&[u8]) + Send  + 'static,
    {
        let (closure, _inner) = Handler::leak_with_fragment_assembler(FnAdapter(handler))?;
        Ok(FragmentHandler { closure, _inner })
    }

    /// Create a subscription bound to an already-built `FragmentHandler`.
    ///
    /// Call this fresh on every (re)connect attempt. It only *borrows*
    /// `handler` (note the `&'a FragmentHandler<F>` param), so it never
    /// competes with `build_fragment_handler` for ownership of whatever
    /// state your closure captured.
    pub fn add_subscription<'a, F>(
        &self,
        channel: &str,
        stream_id: i32,
        handler: &'a FragmentHandler<F>,
    ) -> Result<AeronSubscriber<'a>, SetupError> {
        let disconnected = Arc::new(AtomicBool::new(false));
        // This one small handler IS re-leaked per reconnect attempt — that's
        // fine, it's just an AtomicBool flag tied to this specific
        // subscription's disconnect state, not your message-processing state.
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

        Ok(AeronSubscriber {
            subscription,
            closure: &handler.closure,
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

/// A live subscription for one connection attempt. Borrows its message
/// handler from a `FragmentHandler<F>` built elsewhere (see
/// `AeronTransport::build_fragment_handler`) — that's why this is
/// parameterized by a lifetime `'a` instead of owning `F` directly.
pub struct AeronSubscriber<'a> {
    subscription: AeronSubscription,
    closure: &'a Handler<AeronFragmentAssembler>,
    disconnected: Arc<AtomicBool>,
}

// ── Why `Send + 'static` moved here, and what it fixed ─────────────────────
//
// `Send + 'static` was never really a `Subscriber`-trait requirement — it's
// forced by `Handler::leak_with_fragment_assembler` (see
// `build_fragment_handler` above), which registers a closure with the C-level
// Aeron driver. Once registered, that callback can be invoked by Aeron's
// internals at any time, on any thread, with no Rust lifetime tracking across
// the FFI boundary. So the closure must fully own everything it touches —
// it can't hold a borrow, because a borrow's validity is tied to some Rust
// stack frame Aeron's C code knows nothing about. `'static` is the compiler's
// way of enforcing "this owns everything it needs; nothing it points to can
// disappear."
//
// That's what forced a `move` instead of a borrow in the old design: a
// closure needing `producer` had to capture it *by value* to satisfy
// `'static`. And a move is destructive — it can happen exactly once per
// value. The old `AeronSubscriber<F>` owned its closure directly
// (`_inner: Handler<FnAdapter<F>>`), so a fresh `F` had to be constructed
// every time a new `AeronSubscriber<F>` was built — i.e. on every reconnect
// attempt inside the loop. First iteration: fine, `producer` moves in.
// Any iteration after that: `producer` was already gone → E0382.
//
// The fix here is NOT removing `Send + 'static` — it's still required, just
// relocated to `build_fragment_handler<F>`, called ONCE outside the loop.
// `FragmentHandler<F>` is now the only thing that owns `F`. This
// `AeronSubscriber<'a>` no longer has an `F` type parameter at all — it just
// holds `&'a Handler<AeronFragmentAssembler>`, a plain borrow of something
// already leaked and made `'static`-safe elsewhere. Borrowing is
// non-destructive and repeatable: you can build as many `AeronSubscriber<'a>`
// values as you want from the same `fragment_handler`, on every reconnect,
// without ever moving `producer` (or anything else it captured) again.
impl<'a> Subscriber for AeronSubscriber<'a> {
    type Data = usize;
    type Error = PollError;

    #[inline]
    fn poll(&mut self, fragment_limit: usize) -> Result<Self::Data, Self::Error> {
        let r = self
            .subscription
            .poll(Some(self.closure), fragment_limit)? as usize;
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