use error_stuff::{ErrorDetails, LogCategory, LogLevel, ServerLogEntry};
use server_protocol::Request;
use std::sync::OnceLock;
use tokio::sync::mpsc::UnboundedSender;

/// The id of this server run. Set once at startup, read by every log
/// entry so rows from different runs can be told apart.
static SESSION_ID: OnceLock<String> = OnceLock::new();

/// The channel the executor drains. Anything that wants to write a log
/// row sends a ServerLogEntry here; the executor owns the db, so it's the
/// one that actually inserts.
static LOG_TX: OnceLock<UnboundedSender<ServerLogEntry>> = OnceLock::new();

pub fn install_session_id(id: String) {
    let _ = SESSION_ID.set(id);
}

pub fn install_log_channel(tx: UnboundedSender<ServerLogEntry>) {
    let _ = LOG_TX.set(tx);
}

pub fn timestamp() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!(
        "{:02}:{:02}:{:02}",
        (secs / 3600) % 24,
        (secs / 60) % 60,
        secs % 60
    )
}

pub fn log(label: &str, msg: impl std::fmt::Display) {
    println!("[{}] {}: {}", timestamp(), label, msg);
}

/// Report an error. Prints it with every field it carries, and also sends
/// it down the log channel so the executor can write a row. If the channel
/// isn't installed yet (early startup), it prints only.
///
/// Use this instead of `log` whenever the thing being reported is an
/// error, so the err_msg/file/method don't get lost in a formatted string.
pub fn log_error(label: &str, category: LogCategory, details: &ErrorDetails) {
    print!(
        "[{}] {}: ERROR in {}::{}: {}",
        timestamp(),
        label,
        details.file,
        details.method,
        details.err_msg
    );
    if !details.probable_cause.is_empty() {
        print!(" (probably: {})", details.probable_cause);
    }
    println!();

    let Some(tx) = LOG_TX.get() else {
        return;
    };
    let session_id = SESSION_ID.get().cloned().unwrap_or_default();
    let message = format!(
        "ERROR in {}::{}: {}{}",
        details.file,
        details.method,
        details.err_msg,
        if details.probable_cause.is_empty() {
            String::new()
        } else {
            format!(" (probably: {})", details.probable_cause)
        }
    );
    let entry = ServerLogEntry {
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0),
        level: LogLevel::Error,
        category,
        source: format!("{}::{}", details.file, details.method),
        session_id,
        message,
    };
    let _ = tx.send(entry);
}

pub fn request_name(r: &Request) -> &'static str {
    match r {
        Request::Register => "asked to register",
        Request::Login => "asked to log in",
        Request::SendLoveLetter => "sent a love letter",
        Request::GetAllYrsPages => "asked for all pages",
        Request::Ack => "acknowledged a letter",
    }
}