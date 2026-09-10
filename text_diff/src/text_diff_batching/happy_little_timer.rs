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
    rx: &mpsc::Receiver<TimerInstruction>,
    timer_time: &Arc<Mutex<Timer>>,
    current_state: TickState,
) -> NextStep {
    match rx.try_recv() {
        Ok(TimerInstruction::Reset) => {
            timer_time.lock().unwrap().time_left = TIME_UNTIL_TIMEOUT;
            NextStep::Continue(TickState::CountingDown)
        }
        Ok(TimerInstruction::Cancel) => NextStep::Continue(TickState::Idle),
        Err(mpsc::TryRecvError::Disconnected) => NextStep::Terminate,
        Err(mpsc::TryRecvError::Empty) => NextStep::Continue(current_state),
    }
}

pub fn start(timer_time: Arc<Mutex<Timer>>, rx: mpsc::Receiver<TimerInstruction>) {
    thread::spawn(move || {
        let mut state = TickState::Idle;

        loop {
            wait_patiently(Duration::from_millis(100));

            state = match apply_instruction(&rx, &timer_time, state) {
                NextStep::Continue(new_state) => new_state,
                NextStep::Terminate => return,
            };

            // Act on the current state — the only place ticking happens.
            if let TickState::CountingDown = state {
                let mut timer = timer_time.lock().unwrap();
                timer.time_left -= 0.1;
                if timer.time_left <= 0.0 {
                    state = TickState::Idle;
                }
            }
        }
    });
}

pub fn wait_patiently(time_to_wait: Duration) {
    thread::sleep(time_to_wait);
}
