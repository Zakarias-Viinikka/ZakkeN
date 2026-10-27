//! One writer at a time.
//!
//! Wasm is single-threaded, but tasks interleave at every await. Two
//! writers that both check "is a transaction open?", await, then start
//! one can both see "clear" and both start. This lock closes that
//! look-then-act hole: try_claim does a single atomic
//! compare_exchange, with no await between checking and setting.
//!
//! Every writing entry point takes it. The executors do not take it
//! themselves — they join an open transaction through
//! start_transaction_if_not_in_one!, which is safe because under this
//! lock "a transaction is open" can only mean the current holder
//! opened it.
//!
//! Deadlock rule: never call flush_now() while holding this lock.
//! flush_now() takes it internally. Entry points call flush_now() first,
//! then take the lock for their own executor.

use std::sync::LazyLock;
use std::sync::atomic::{AtomicBool, Ordering};

use gloo_timers::future::TimeoutFuture;

pub static WRITE_LOCK: LazyLock<WriteLock> = LazyLock::new(WriteLock::new);

pub struct WriteLock {
    held: AtomicBool,
}

impl WriteLock {
    const fn new() -> Self {
        Self {
            held: AtomicBool::new(false),
        }
    }

    fn try_claim(&self) -> bool {
        self.held
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }

    fn release(&self) {
        self.held.store(false, Ordering::Release);
    }

    pub async fn acquire(&self) -> WriteGuard<'_> {
        while !self.try_claim() {
            TimeoutFuture::new(1).await;
        }
        WriteGuard { lock: self }
    }
}

pub struct WriteGuard<'a> {
    lock: &'a WriteLock,
}

impl<'a> Drop for WriteGuard<'a> {
    fn drop(&mut self) {
        self.lock.release();
    }
}