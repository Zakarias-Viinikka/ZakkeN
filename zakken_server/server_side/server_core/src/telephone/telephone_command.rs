/// A message the telephone sends to one socket. The socket listens on
/// one channel for these; the value tells it what to do next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TelephoneCommand {
    /// Fetch this user's unsent letters and push each one to the
    /// client.
    GoPullYourUnsentLetters,
    /// The drift version just changed and this socket is stale. Ask
    /// the executor for the current one, tell the client, then close.
    TellTheClientTheDriftVersionChangedThenClose,
}
