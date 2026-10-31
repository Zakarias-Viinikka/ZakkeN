use leptos::prelude::*;
use leptos::task::spawn_local;
use std::time::Duration;

use crate::shared_owners::power::POWERED;

pub const LATENCY_LIGHT_MS: u64 = 500;
pub const LATENCY_HEAVY_MS: u64 = 900;

pub fn reply_after_delay(ms: u64, f: impl FnOnce() + 'static) {
    let (tx, rx) = futures::channel::oneshot::channel::<()>();
    leptos::prelude::set_timeout(
        move || {
            let _ = tx.send(());
        },
        Duration::from_millis(ms),
    );
    spawn_local(async move {
        let _ = rx.await;
        f();
    });
}

pub fn reply_after_delay_if_still_powered(ms: u64, f: impl FnOnce() + 'static) {
    let (tx, rx) = futures::channel::oneshot::channel::<()>();
    leptos::prelude::set_timeout(
        move || {
            let _ = tx.send(());
        },
        Duration::from_millis(ms),
    );
    let powered_sig = POWERED.clone();
    spawn_local(async move {
        let _ = rx.await;
        if powered_sig.get_untracked() {
            f();
        }
    });
}