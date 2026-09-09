use std::time::Duration;

/// Waits for a platform-dependent amount of time and returns a platform-specific string.
pub async fn start_timer() -> String {
    let time_to_wait_secs: f64 = get_time_to_wait();
    let duration = Duration::from_secs_f64(time_to_wait_secs);

    do_the_wait_and_get_the_string_after(duration).await
}

pub async fn do_the_wait_and_get_the_string_after(duration: Duration) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        wasm_timer(duration).await
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        normal_timer(duration).await
    }
}

fn get_time_to_wait() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        2.0
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        0.5
    }
}

/// Native timer implementation (uses futures-timer).
#[cfg(not(target_arch = "wasm32"))]
pub async fn normal_timer(duration: Duration) -> String {
    futures_timer::Delay::new(duration).await;
    "hello world".to_string()
}

/// Wasm timer implementation (uses gloo-timers).
#[cfg(target_arch = "wasm32")]
pub async fn wasm_timer(duration: Duration) -> String {
    gloo_timers::future::TimeoutFuture::new(duration.as_millis() as u32).await;
    "hello web".to_string()
}
