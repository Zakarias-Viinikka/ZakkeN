use crate::yrs_error::{DeadlockPrediction, ErrorInfo, YrsError};
use std::sync::{Once, mpsc};
use std::thread;
use std::time::Duration;

const DEBUG_MODE: bool = true;
const DEFAULT_TIME_UNTILL_THROW_DEADLOCK_ERROR: Duration = Duration::from_secs(10);
const DBG_TIME_UNTILL_THROW_DEADLOCK_ERROR: Duration = Duration::from_secs(2);

static INIT: Once = Once::new();

pub fn init_debug_message() {
    INIT.call_once(|| {
        if DEBUG_MODE {
            println!("anti_deadlock: DEBUG_MODE is ON (thread + timeout enabled)");
        } else {
            println!("anti_deadlock: DEBUG_MODE is OFF (direct call, no timeout)");
        }
    });
}

pub struct DeadlockCtx {
    pub method_name: &'static str,
    pub file_name: &'static str,
    pub timeout: DurationSettings,
    pub prediction: DeadlockPrediction,
}

impl DeadlockCtx {
    pub fn new(
        method_name: &'static str,
        file_name: &'static str,
        prediction: DeadlockPrediction,
    ) -> Self {
        Self {
            method_name,
            file_name,
            timeout: DurationSettings::Default,
            prediction,
        }
    }

    pub fn with_timeout(mut self, timeout: DurationSettings) -> Self {
        self.timeout = timeout;
        self
    }
}

pub enum DurationSettings {
    Default,
    Milliseconds(u64),
    Seconds(u64),
}

impl DurationSettings {
    fn to_duration(&self) -> Duration {
        match self {
            DurationSettings::Default => {
                if DEBUG_MODE {
                    DBG_TIME_UNTILL_THROW_DEADLOCK_ERROR
                } else {
                    DEFAULT_TIME_UNTILL_THROW_DEADLOCK_ERROR
                }
            }
            DurationSettings::Milliseconds(ms) => Duration::from_millis(*ms),
            DurationSettings::Seconds(s) => Duration::from_secs(*s),
        }
    }
}

pub fn prevent_deadlock<F, T>(ctx: DeadlockCtx, callback: F) -> Result<T, YrsError>
where
    F: FnOnce() -> Result<T, YrsError> + Send + 'static,
    T: Send + 'static,
{
    init_debug_message();

    if !DEBUG_MODE {
        return callback();
    }

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let result = callback();
        let _ = tx.send(result);
    });

    let timeout = ctx.timeout.to_duration();

    match rx.recv_timeout(timeout) {
        Ok(result) => result,
        Err(mpsc::RecvTimeoutError::Timeout) => Err(YrsError::Deadlock {
            prediction: ctx.prediction,
            info: ErrorInfo {
                error_msg: "deadlock detected".to_string(),
                file: ctx.file_name.to_string(),
                method: ctx.method_name.to_string(),
            },
        }),
        Err(mpsc::RecvTimeoutError::Disconnected) => Err(YrsError::GenericError {
            info: ErrorInfo {
                error_msg: "worker thread panicked or disconnected".to_string(),
                file: ctx.file_name.to_string(),
                method: ctx.method_name.to_string(),
            },
        }),
    }
}
