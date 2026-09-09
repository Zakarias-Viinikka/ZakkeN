use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;

use crate::text_diff_batching::happy_little_timer::{self, Timer, TimerInstruction};

pub struct DiffTimer {
    tx: mpsc::Sender<Signal>,
}

enum Signal {
    Reset,
    Cancel,
    FireAndReset,
    FireAndClear,
    Shutdown,
}

enum JobSecurity {
    StillGotAJob,
    Fired,
}

impl DiffTimer {
    pub fn new(on_timeout: impl Fn() + Send + Sync + 'static) -> Self {
        let (tx, rx) = mpsc::channel::<Signal>();
        let (timer_tx, timer_rx) = mpsc::channel::<TimerInstruction>();
        let timer_arc = Arc::new(Mutex::new(Timer::new()));

        // Start the actual timer thread
        happy_little_timer::start(Arc::clone(&timer_arc), timer_rx);

        thread::spawn(move || {
            let receiver = rx;
            let callback = on_timeout;

            loop {
                match wait_for_instructions(&receiver, &callback) {
                    JobSecurity::StillGotAJob => {
                        match run_countdown_loop(&receiver, &callback, &timer_tx, &timer_arc) {
                            JobSecurity::StillGotAJob => continue,
                            JobSecurity::Fired => break,
                        }
                    }
                    JobSecurity::Fired => break,
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

    pub fn fire_and_reset(&self) {
        let _ = self.tx.send(Signal::FireAndReset);
    }

    pub fn fire_and_clear(&self) {
        let _ = self.tx.send(Signal::FireAndClear);
    }
}

fn wait_for_instructions<F>(rx: &mpsc::Receiver<Signal>, on_timeout: &F) -> JobSecurity
where
    F: Fn() + Send + Sync,
{
    loop {
        match rx.recv() {
            Ok(Signal::Reset) => return JobSecurity::StillGotAJob,
            Ok(Signal::Cancel) => continue,
            Ok(Signal::FireAndReset) => {
                on_timeout();
                return JobSecurity::StillGotAJob;
            }
            Ok(Signal::FireAndClear) => {
                on_timeout();
                continue;
            }
            Ok(Signal::Shutdown) | Err(_) => return JobSecurity::Fired,
        }
    }
}

fn run_countdown_loop<F>(
    rx: &mpsc::Receiver<Signal>,
    on_timeout: &F,
    timer_tx: &mpsc::Sender<TimerInstruction>,
    timer_arc: &Arc<Mutex<Timer>>,
) -> JobSecurity
where
    F: Fn() + Send + Sync,
{
    loop {
        // Wait a short interval before checking again
        happy_little_timer::wait_patiently(Duration::from_millis(50));

        // Check for any pending commands (non‑blocking)
        match rx.try_recv() {
            Ok(Signal::Reset) => {
                let _ = timer_tx.send(TimerInstruction::Reset);
                continue;
            }
            Ok(Signal::Cancel) => {
                let _ = timer_tx.send(TimerInstruction::Cancel);
                return JobSecurity::StillGotAJob;
            }
            Ok(Signal::FireAndReset) => {
                on_timeout();
                let _ = timer_tx.send(TimerInstruction::Reset);
                continue;
            }
            Ok(Signal::FireAndClear) => {
                on_timeout();
                let _ = timer_tx.send(TimerInstruction::Cancel);
                return JobSecurity::StillGotAJob;
            }
            Ok(Signal::Shutdown) => return JobSecurity::Fired,
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => return JobSecurity::Fired,
        }

        // Check if the timer has finished
        let timer = timer_arc.lock().unwrap();
        if timer.time_left <= 0.0 {
            on_timeout();
            return JobSecurity::StillGotAJob;
        }
    }
}

impl Drop for DiffTimer {
    fn drop(&mut self) {
        let _ = self.tx.send(Signal::Shutdown);
    }
}
