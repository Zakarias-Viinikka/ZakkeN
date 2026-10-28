use std::fmt;

use smart_frac_index::frac_core_internals::errors::FracIndexError;

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct ErrorInfo {
    pub error_msg: String,
    pub file: String,
    pub method: String,
}

impl fmt::Display for ErrorInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (file: {}, method: {})",
            self.error_msg, self.file, self.method
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum DeadlockPrediction {
    PotentiallyJustSlowOperation(String),
    ProbablyJustADeadlock,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Error)]
pub enum YrsError {
    GenericError {
        info: ErrorInfo,
    },
    YrsInternalError {
        info: ErrorInfo,
    },
    Deadlock {
        prediction: DeadlockPrediction,
        info: ErrorInfo,
    },
}

impl fmt::Display for YrsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            YrsError::GenericError { info } => write!(f, "Generic error: {}", info),
            YrsError::YrsInternalError { info } => write!(f, "Yrs internal error: {}", info),
            YrsError::Deadlock { prediction, info } => {
                write!(f, "Deadlock predicted as {:?} (info: {})", prediction, info)
            }
        }
    }
}

impl From<FracIndexError> for YrsError {
    fn from(e: FracIndexError) -> Self {
        match e {
            FracIndexError::GenericError { info } => YrsError::GenericError {
                info: ErrorInfo {
                    error_msg: info.error_msg,
                    file: info.file,
                    method: info.method,
                },
            },
        }
    }
}

impl std::error::Error for YrsError {}