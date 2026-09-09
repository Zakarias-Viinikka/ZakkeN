use std::{
    sync::{Arc, Mutex, mpsc},
    thread,
    time::Duration,
};

const TIME_UNTIL_TIMEOUT: f32 = 0.5; // seconds

pub struct Timer {
    pub time_left: f32,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            time_left: TIME_UNTIL_TIMEOUT,
        }
    }
}

pub enum TimerInstruction {
    Reset,
    Cancel,
}

pub fn start(timer_time: Arc<Mutex<Timer>>, rx: mpsc::Receiver<TimerInstruction>) {
    thread::spawn(move || {
        loop {
            wait_patiently(Duration::from_millis(100));

            // Check for any instruction without blocking
            match rx.try_recv() {
                Ok(TimerInstruction::Cancel) | Err(mpsc::TryRecvError::Disconnected) => {
                    break;
                }
                Ok(TimerInstruction::Reset) => {
                    timer_time.lock().unwrap().time_left = TIME_UNTIL_TIMEOUT;
                    continue;
                }
                Err(mpsc::TryRecvError::Empty) => {}
            }

            // Decrement time_left
            let mut timer = timer_time.lock().unwrap();
            timer.time_left -= 0.1;
            if timer.time_left <= 0.0 {
                break;
            }
        }
    });
}

#[cfg(not(target_arch = "wasm32"))]
pub fn wait_patiently(time_to_wait: Duration) {
    std::thread::sleep(time_to_wait);
}

#[cfg(target_arch = "wasm32")]
pub async fn wait_patiently(time_to_wait: Duration) {
    gloo_timers::future::TimeoutFuture::new(time_to_wait.as_millis() as u32).await;
}
