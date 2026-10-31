//! Asks the OS for an unused port by binding to 127.0.0.1:0 and reading
//! back what it chose. The listener drops immediately; the caller reuses
//! the port. Small race window; fine for local dev.

pub fn main_for_finding_a_free_port() -> Result<u16, String> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")
        .map_err(|e| format!("could not bind for free port: {e}"))?;
    let port = listener
        .local_addr()
        .map_err(|e| format!("could not read local addr: {e}"))?
        .port();
    drop(listener);
    Ok(port)
}
