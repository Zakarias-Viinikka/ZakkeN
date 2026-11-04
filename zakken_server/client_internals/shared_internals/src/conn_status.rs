#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnStatus {
    /// Attempting to open the socket, or retrying after a failure.
    Connecting,
    /// Socket is open.
    Connected,
    /// A connection attempt failed. Will retry with backoff.
    Failed,
}
