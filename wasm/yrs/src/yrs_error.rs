use std::fmt;

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
    Deadlock {
        prediction: DeadlockPrediction,
        info: ErrorInfo,
    },
}

impl fmt::Display for YrsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            YrsError::GenericError { info } => write!(f, "Generic error: {}", info),
            YrsError::Deadlock { prediction, info } => {
                write!(f, "Deadlock predicted as {:?} (info: {})", prediction, info)
            }
        }
    }
}

impl std::error::Error for YrsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
