#[derive(Debug, Clone, uniffi::Record)]
pub struct ErrorInfo {
    pub error_msg: String,
    pub file: String,
    pub method: String,
}

#[derive(Debug, Clone, uniffi::Error, thiserror::Error)]
pub enum FracIndexError {
    #[error("{} ({}, {})", info.error_msg, info.method, info.file)]
    GenericError { info: ErrorInfo },
}

pub(crate) fn err(msg: impl Into<String>, method: &str) -> FracIndexError {
    FracIndexError::GenericError {
        info: ErrorInfo {
            error_msg: msg.into(),
            file: file!().to_string(),
            method: method.to_string(),
        },
    }
}
