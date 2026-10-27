//! "Only one tab connects at a time" via the Web Locks API.
//!
//! Called through js_sys::Reflect so the code doesn't need web-sys
//! unstable features.
//!
//! Behaviour we rely on:
//! - `ifAvailable: true`: the outer promise resolves `null` immediately
//!   if the lock is already held by another tab.
//! - If we DO get the lock, the callback's promise is returned to the
//!   Web Locks spec. Our callback returns a never-resolving Promise,
//!   so the outer promise stays pending forever.
//!
//! So: race the outer promise against a small timeout.
//! - Resolved (any value): we didn't get the lock. Return false.
//! - Still pending after the timeout: we hold the lock. Return true.

use futures_util::future::Either;
use gloo_timers::future::TimeoutFuture;
use js_sys::{Function, Object, Promise, Reflect};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

const LOCK_NAME: &str = "zakken_server_link";

pub async fn acquire() -> bool {
    let Some(window) = web_sys::window() else {
        return true;
    };
    let navigator = window.navigator();
    let Ok(locks) = Reflect::get(navigator.as_ref(), &JsValue::from_str("locks")) else {
        return true;
    };
    if locks.is_null() || locks.is_undefined() {
        return true;
    }
    let Ok(request_val) = Reflect::get(&locks, &JsValue::from_str("request")) else {
        return true;
    };
    let Ok(request_fn) = request_val.dyn_into::<Function>() else {
        return true;
    };

    // Never-resolving promise: as long as this is pending, the lock is held.
    let hold = Promise::new(&mut |_resolve, _reject| {});

    let callback = {
        let hold = hold.clone();
        Closure::once_into_js(move || -> JsValue { hold.into() })
    };

    let opts = Object::new();
    let _ = Reflect::set(&opts, &JsValue::from_str("ifAvailable"), &JsValue::TRUE);

    let args = js_sys::Array::new();
    args.push(&JsValue::from_str(LOCK_NAME));
    args.push(&opts);
    args.push(&callback);

    let outer: Promise = match request_fn.apply(&locks, &args) {
        Ok(v) if v.is_instance_of::<Promise>() => v.unchecked_into(),
        _ => return true,
    };

    let promise_fut = JsFuture::from(outer);
    let timeout_fut = TimeoutFuture::new(200);
    futures_util::pin_mut!(promise_fut, timeout_fut);

    match futures_util::future::select(promise_fut, timeout_fut).await {
        Either::Left((_resolved, _)) => {
            // Outer promise resolved. With our never-resolving callback,
            // that means we didn't get the lock.
            false
        }
        Either::Right((_, _)) => {
            // Outer promise still pending. We hold the lock.
            // Leak `hold` so the JS ref count on the promise stays up.
            std::mem::forget(hold);
            true
        }
    }
}