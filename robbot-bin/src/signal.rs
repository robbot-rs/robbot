use futures::future::BoxFuture;
use tokio::sync::watch;

use std::future::Future;
use std::pin::Pin;
use std::ptr;
use std::sync::Once;
use std::task::{Context, Poll};

static TERM_ONCE: Once = Once::new();
static mut TERM_TX: *const watch::Sender<bool> = ptr::null();

/// Subscribe to the shutdown signal. Returns an ShutdownListener whose Future
/// will complete when a shutdown signal is received.
pub fn subscribe<'a>() -> ShutdownListener<'a> {
    TERM_ONCE.call_once(|| {
        let (tx, _) = watch::channel(false);

        unsafe {
            TERM_TX = Box::leak(Box::new(tx)) as *const _;
        }
    });

    // `TERM_TX` cannot be null, `watch::Sender<bool>` is Send + Sync.
    let tx = unsafe { &*TERM_TX };

    ShutdownListener::new(tx.subscribe())
}

/// Sends a signal to terminate to all [`ShutdownListener`]s, prompting them
/// to shut down. A `terminate()` call cannot be undone.
pub fn terminate() {
    // Skip terminate call, `TERM_TX` is null, noone is listening.
    if !TERM_ONCE.is_completed() {
        return;
    }

    let tx = unsafe { &*TERM_TX };
    let _ = tx.send(true);
}

/// A listener for shutdown signals. When a shutdown signal is received, the
/// `ShutdownListeners` future will complete.
pub struct ShutdownListener<'a> {
    rx: watch::Receiver<bool>,
    fut: Option<BoxFuture<'a, ()>>,
}

impl<'a> ShutdownListener<'a> {
    /// Creates a new `ShutdownListener` from a receiver.
    fn new(rx: watch::Receiver<bool>) -> Self {
        Self { rx, fut: None }
    }
}

impl<'a> Future for ShutdownListener<'a> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Self::Output> {
        if self.fut.is_none() {
            let mut rx = self.rx.clone();
            self.fut = Some(Box::pin(async move {
                let _ = rx.changed().await;
            }));
        }

        self.fut.as_mut().unwrap().as_mut().poll(ctx)
    }
}

/// Registers all signal handlers. Before `init` is called, no signal are intercepted.
/// The exact behavior depend on the os family.
pub fn init() {
    #[cfg(target_family = "unix")]
    unix::init();
}

#[cfg(target_family = "unix")]
mod unix {
    use super::terminate;
    use nix::sys::signal::{sigaction, SaFlags, SigAction, SigHandler, SigSet, Signal};

    /// Registers signal handlers for the following signals:
    /// - SIGINT
    /// - SIGTERM
    pub(super) fn init() {
        let term = SigAction::new(
            SigHandler::Handler(handle_sigterm),
            SaFlags::empty(),
            SigSet::empty(),
        );

        let int = SigAction::new(
            SigHandler::Handler(handle_sigint),
            SaFlags::empty(),
            SigSet::empty(),
        );

        // SAFETY: No previous handler was installed from an external source and no
        // syscalls are made in the `SigHandler` body.
        unsafe {
            sigaction(Signal::SIGTERM, &term).unwrap();
            sigaction(Signal::SIGINT, &int).unwrap();
        }
    }

    /// The handler for the SIGINT signal.
    extern "C" fn handle_sigint(_: i32) {
        terminate();
    }

    /// The handler for the SIGTERM signal.
    extern "C" fn handle_sigterm(_: i32) {
        terminate();
    }
}
