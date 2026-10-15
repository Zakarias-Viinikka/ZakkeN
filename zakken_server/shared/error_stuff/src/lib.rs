use protocol::error::DbError;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum ServErr {
    #[error("db error: {0}")]
    DbErrorContainer(DbError),
    #[error("{0}")]
    ServerError(ErrorDetails),
    #[error("client '{user_name}' is gone")]
    ClientGone { user_name: String },
}

impl From<DbError> for ServErr {
    fn from(e: DbError) -> Self {
        ServErr::DbErrorContainer(e)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetails {
    pub err_msg: String,
    pub file: String,
    pub method: String,
    /// A guess at why this happened, when the cause is easy to name.
    /// Empty string means "no guess".
    pub probable_cause: String,
}

impl fmt::Display for ErrorDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "server error in {}::{}: {}",
            self.file, self.method, self.err_msg
        )?;
        if !self.probable_cause.is_empty() {
            write!(f, " (probably: {})", self.probable_cause)?;
        }
        Ok(())
    }
}

pub mod server_internal_error_logging;
