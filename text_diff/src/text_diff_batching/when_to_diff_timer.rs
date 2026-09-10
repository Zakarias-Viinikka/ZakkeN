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

enum CountdownState {
    Idle,
    CountingDown,
}

enum NextStep {
    Continue(CountdownState),
    Terminate,
}

/// Reads at most one pending signal, applies its side effects (firing the
/// callback, forwarding Reset/Cancel to the inner timer), and decides what
/// state comes next.
fn apply_signal<F: Fn()>(
    rx: &mpsc::Receiver<Signal>,
    timer_tx: &mpsc::Sender<TimerInstruction>,
    on_timeout: &F,
    current_state: CountdownState,
) -> NextStep {
    match rx.try_recv() {
        Ok(Signal::Reset) => {
            let _ = timer_tx.send(TimerInstruction::Reset);
            NextStep::Continue(CountdownState::CountingDown)
        }
        Ok(Signal::Cancel) => {
            let _ = timer_tx.send(TimerInstruction::Cancel);
            NextStep::Continue(CountdownState::Idle)
        }
        Ok(Signal::FireAndReset) => {
            on_timeout();
            let _ = timer_tx.send(TimerInstruction::Reset);
            NextStep::Continue(CountdownState::CountingDown)
        }
        Ok(Signal::FireAndClear) => {
            on_timeout();
            let _ = timer_tx.send(TimerInstruction::Cancel);
            NextStep::Continue(CountdownState::Idle)
        }
        Ok(Signal::Shutdown) => NextStep::Terminate,
        Err(mpsc::TryRecvError::Disconnected) => NextStep::Terminate,
        Err(mpsc::TryRecvError::Empty) => NextStep::Continue(current_state),
    }
}

impl DiffTimer {
    pub fn new(on_timeout: impl Fn() + Send + Sync + 'static) -> Self {
        let (tx, rx) = mpsc::channel::<Signal>();
        let (timer_tx, timer_rx) = mpsc::channel::<TimerInstruction>();
        let timer_arc = Arc::new(Mutex::new(Timer::new()));

        happy_little_timer::start(Arc::clone(&timer_arc), timer_rx);

        thread::spawn(move || {
            let mut state = CountdownState::Idle;

            loop {
                happy_little_timer::wait_patiently(Duration::from_millis(50));

                state = match apply_signal(&rx, &timer_tx, &on_timeout, state) {
                    NextStep::Continue(new_state) => new_state,
                    NextStep::Terminate => return,
                };

                // Act on the current state — the only place expiry is checked.
                if let CountdownState::CountingDown = state {
                    let expired = timer_arc.lock().unwrap().time_left <= 0.0;
                    if expired {
                        on_timeout();
                        state = CountdownState::Idle;
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

    pub fn fire_and_reset(&self) {
        let _ = self.tx.send(Signal::FireAndReset);
    }

    pub fn fire_and_clear(&self) {
        let _ = self.tx.send(Signal::FireAndClear);
    }
}

impl Drop for DiffTimer {
    fn drop(&mut self) {
        let _ = self.tx.send(Signal::Shutdown);
    }
}
