use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use futures::channel::mpsc;

use crate::text_diff_batching::batch_text_edits::DiffCallback;

const TIME_UNTIL_TIMEOUT: f32 = 0.5; // seconds

pub struct HappyLittleTimer {
    pub time_left: f32,
    notify_somebody: Arc<dyn DiffCallback>,
}

impl HappyLittleTimer {
    pub fn new(callback: Arc<dyn DiffCallback>) -> Self {
        Self {
            time_left: TIME_UNTIL_TIMEOUT,
            notify_somebody: callback,
        }
    }
}

pub enum TimerInstruction {
    Reset,
    Cancel,
}

enum TickState {
    Idle,
    CountingDown,
}

enum NextStep {
    Continue(TickState),
    Terminate,
}

/// Reads at most one pending instruction and decides what state comes next.
fn apply_instruction(
    rx: &mut mpsc::Receiver<TimerInstruction>,
    self_wrapped_in_arc_mutex: &Arc<Mutex<HappyLittleTimer>>,
    current_state: TickState,
) -> NextStep {
    match rx.try_next() {
        Ok(Some(TimerInstruction::Reset)) => {
            self_wrapped_in_arc_mutex.lock().unwrap().time_left = TIME_UNTIL_TIMEOUT;
            NextStep::Continue(TickState::CountingDown)
        }
        Ok(Some(TimerInstruction::Cancel)) => NextStep::Continue(TickState::Idle),
        Ok(None) => NextStep::Terminate,
        Err(_) => NextStep::Continue(current_state),
    }
}

pub fn start(
    self_wrapped_in_arc_mutex: Arc<Mutex<HappyLittleTimer>>,
    mut rx: mpsc::Receiver<TimerInstruction>,
) {
    thread::spawn(move || {
        let mut state = TickState::Idle;

        loop {
            wait_patiently(Duration::from_millis(100));

            state = match apply_instruction(&mut rx, &self_wrapped_in_arc_mutex, state) {
                NextStep::Continue(new_state) => new_state,
                NextStep::Terminate => return,
            };

            // Act on the current state — the only place ticking happens.
            if let TickState::CountingDown = state {
                let mut timer = self_wrapped_in_arc_mutex.lock().unwrap().time_left;
                timer -= 0.1;
                if timer <= 0.0 {
                    self_wrapped_in_arc_mutex
                        .lock()
                        .unwrap()
                        .notify_somebody
                        .on_diff_needed();
                    state = TickState::Idle;
                }
            }
        }
    });
}

pub fn wait_patiently(time_to_wait: Duration) {
    thread::sleep(time_to_wait);
}
