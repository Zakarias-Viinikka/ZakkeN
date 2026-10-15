use serde::{Deserialize, Serialize};

/// The coarse area of the server a log entry belongs to.
/// Use it to filter: "show me every error from letter delivery".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogCategory {
    /// Sockets: connecting, disconnecting, login and register.
    Connection,
    /// Delivering letters: fan-out, unsent rows, acks.
    LetterDelivery,
    /// Anything that failed while talking to sqlite.
    Database,
    /// The request queue or the queue executor.
    Queue,
    /// Errors that happened while writing a log entry.
    /// Yes, the log system logging its own failure.
    Logging,
    /// Startup, shutdown, and anything that doesn't fit above.
    Server,
}

impl LogCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            LogCategory::Connection => "CONNECTION",
            LogCategory::LetterDelivery => "LETTER_DELIVERY",
            LogCategory::Database => "DATABASE",
            LogCategory::Queue => "QUEUE",
            LogCategory::Logging => "LOGGING",
            LogCategory::Server => "SERVER",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "CONNECTION" => Some(LogCategory::Connection),
            "LETTER_DELIVERY" => Some(LogCategory::LetterDelivery),
            "DATABASE" => Some(LogCategory::Database),
            "QUEUE" => Some(LogCategory::Queue),
            "LOGGING" => Some(LogCategory::Logging),
            "SERVER" => Some(LogCategory::Server),
            _ => None,
        }
    }
}

/// How bad the entry is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    /// Normal operation, worth keeping a record of.
    Info,
    /// Something recoverable went wrong, or a client vanished.
    Warn,
    /// An operation failed but the server is still running.
    Error,
    /// The server can't keep doing its job.
    /// Something is broken badly enough that it needs a human.
    CriticalError,
}

impl LogLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::CriticalError => "CRITICAL_ERROR",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "INFO" => Some(LogLevel::Info),
            "WARN" => Some(LogLevel::Warn),
            "ERROR" => Some(LogLevel::Error),
            "CRITICAL_ERROR" => Some(LogLevel::CriticalError),
            _ => None,
        }
    }
}

/// One row for the `logs` table, plus a phrase to print.
///
/// This is the message connected_clients sends when it needs to log.
/// The executor is the only thing that owns the db, so logging goes
/// through the queue like everything else: main wires a channel from
/// whatever wants to log to the executor, which writes the row.
#[derive(Debug, Clone)]
pub struct ServerLogEntry {
    /// Unix seconds.
    pub timestamp: i64,
    pub level: LogLevel,
    pub category: LogCategory,
    /// The finer origin inside the category, like
    /// "send_love_letter.rs::handle".
    pub source: String,
    /// Which run of the server this came from.
    pub session_id: String,
    /// Human-readable text: what you'd read in the terminal. For errors,
    /// this carries err_msg/file/method/probable_cause inside it, so
    /// there's no separate details blob.
    pub message: String,
}

impl ServerLogEntry {
    pub fn now(
        level: LogLevel,
        category: LogCategory,
        source: impl Into<String>,
        session_id: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        Self {
            timestamp,
            level,
            category,
            source: source.into(),
            session_id: session_id.into(),
            message: message.into(),
        }
    }
}
