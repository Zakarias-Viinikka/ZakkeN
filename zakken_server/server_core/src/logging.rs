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

pub fn request_name(r: &Request) -> &'static str {
    match r {
        Request::Register => "asked to register",
        Request::Login => "asked to log in",
        Request::SendLoveLetter => "sent a love letter",
        Request::GetAllYrsPages => "asked for all pages",
    }
}