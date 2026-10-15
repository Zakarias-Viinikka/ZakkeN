use error_stuff::ErrorDetails;
use server_protocol::Request;

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

/// Print an error with every field it carries: what went wrong, where,
/// which function, and a guess at why. Use this instead of `log` whenever
/// the thing being reported is an error, so the err_msg/file/method don't
/// get lost in a formatted string.
pub fn log_error(label: &str, details: &ErrorDetails) {
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