use std::time::Duration;

/// A resettable, one-shot-per-fire countdown timer.
///
/// Idle until the first `reset()`. Each `reset()` (re)starts the countdown
/// from `timeout`. `cancel()` aborts any pending countdown and goes back to
/// idle. On expiry, `on_timeout` fires once and the timer returns to idle
/// until the next `reset()`.
///
/// Two backends live behind this same API: native/Android uses a real OS
/// thread (wasm32 has none — `std::thread::spawn` panics there), wasm uses
/// the browser's setTimeout via `gloo_timers`.
pub struct WhenToDiffTimer {
    inner: PlatformTimer,
}

impl WhenToDiffTimer {
    pub fn new(timeout: Duration, on_timeout: impl Fn() + Send + Sync + 'static) -> Self {
        Self {
            inner: PlatformTimer::new(timeout, on_timeout),
        }
    }

    pub fn reset(&self) {
        self.inner.reset();
    }

    pub fn cancel(&self) {
        self.inner.cancel();
    }
}

#[cfg(not(target_arch = "wasm32"))]
use native_timer::PlatformTimer;

#[cfg(target_arch = "wasm32")]
use wasm_timer::PlatformTimer;

#[cfg(not(target_arch = "wasm32"))]
mod native_timer {
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    enum Signal {
        Reset,
        Cancel,
        Shutdown,
    }

    pub struct PlatformTimer {
        tx: mpsc::Sender<Signal>,
    }

    impl PlatformTimer {
        pub fn new(timeout: Duration, on_timeout: impl Fn() + Send + Sync + 'static) -> Self {
            let (tx, rx) = mpsc::channel::<Signal>();

            thread::spawn(move || {
                loop {
                    // Idle: wait for the first reset to start counting down.
                    match rx.recv() {
                        Ok(Signal::Reset) => {}
                        Ok(Signal::Cancel) => continue, // already idle, no-op
                        Ok(Signal::Shutdown) | Err(_) => return,
                    }

                    // Counting down since the last reset.
                    loop {
                        match rx.recv_timeout(timeout) {
                            Ok(Signal::Reset) => continue, // restart countdown
                            Ok(Signal::Cancel) => break,   // back to idle
                            Ok(Signal::Shutdown) => return,
                            Err(mpsc::RecvTimeoutError::Timeout) => {
                                on_timeout();
                                break; // fired, back to idle
                            }
                            Err(mpsc::RecvTimeoutError::Disconnected) => return,
                        }
                    }
                }
            });

            Self { tx }
        }

        pub fn reset(&self) {
            let _ = self.tx.send(Signal::Reset);
        }

        pub fn cancel(&self) {
            let _ = self.tx.send(Signal::Cancel);
        }
    }

    impl Drop for PlatformTimer {
        fn drop(&mut self) {
            let _ = self.tx.send(Signal::Shutdown);
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm_timer {
    use gloo_timers::callback::Timeout;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::time::Duration;

    pub struct PlatformTimer {
        timeout_ms: u32,
        on_timeout: Rc<dyn Fn()>,
        // Holding the handle keeps it alive; dropping/replacing it cancels
        // the pending browser timeout automatically.
        pending: RefCell<Option<Timeout>>,
    }

    impl PlatformTimer {
        pub fn new(timeout: Duration, on_timeout: impl Fn() + Send + Sync + 'static) -> Self {
            Self {
                timeout_ms: timeout.as_millis() as u32,
                on_timeout: Rc::new(on_timeout),
                pending: RefCell::new(None),
            }
        }

        pub fn reset(&self) {
            let callback = Rc::clone(&self.on_timeout);
            let ms = self.timeout_ms;
            let handle = Timeout::new(ms, move || callback());
            // Assigning over the old Some(handle) drops (cancels) it first.
            *self.pending.borrow_mut() = Some(handle);
        }

        pub fn cancel(&self) {
            self.pending.borrow_mut().take(); // drop = clearTimeout
        }
    }
}
