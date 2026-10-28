use crate::yrs_error::{ErrorInfo, YrsError};

#[macro_export]
macro_rules! generic_error {
    ($msg:expr, $method:expr) => {
        $crate::yrs_error::YrsError::GenericError {
            info: $crate::yrs_error::ErrorInfo {
                error_msg: ::std::string::ToString::to_string(&$msg),
                file: file!().to_string(),
                method: $method.to_string(),
            },
        }
    };
}

#[macro_export]
macro_rules! lock_poisoned {
    ($method:expr) => {
        $crate::generic_error!("lock poisoned", $method)
    };
}

#[macro_export]
macro_rules! read_doc {
    ($doc:expr, $method:expr) => {
        $doc.read().map_err(|_| $crate::lock_poisoned!($method))?
    };
}

#[macro_export]
macro_rules! write_doc {
    ($doc:expr, $method:expr) => {
        $doc.write().map_err(|_| $crate::lock_poisoned!($method))?
    };
}

#[macro_export]
macro_rules! guarded {
    ($method:expr, $work:expr) => {
        $crate::anti_deadlock::prevent_deadlock(
            $crate::anti_deadlock::DeadlockCtx::new(
                $method,
                file!(),
                $crate::yrs_error::DeadlockPrediction::ProbablyJustADeadlock,
            ),
            $work,
        )
    };
}

pub fn internal_error(error_msg: impl Into<String>, method: &str) -> YrsError {
    YrsError::YrsInternalError {
        info: ErrorInfo {
            error_msg: error_msg.into(),
            file: file!().to_string(),
            method: method.to_string(),
        }
    }
}
