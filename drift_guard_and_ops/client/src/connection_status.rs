#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConnectionStatus {
    Offline,
    Connecting(&'static str),
    /// Socket is open but login and the drift comparison haven't
    /// finished yet. Nothing may be sent from here.
    LoggingIn,
    /// Either the in-between step between connecting and failed to connect due to
    /// failing to connect in general. But it's also the state for when the client
    /// tries to log in with x username and it fails for whatever reason.
    FailedToConnect,
    /// Currently no support for the same browser having two tabs connected to server at once, would be complicated to fix
    TwoTabs,
    Connected,
    LonewolfAndOffline,
    /// Means connected to the server, but sync is disabled for whatever reason.
    /// Either because the user has been forced into lonewolf due to another user
    /// forcing the server to a new dataset. Or it happens because the client
    /// deliberately unsynced. It's also the state the client ends up in due to
    /// specific error handling — for example, if the client tries to push a love
    /// letter to the server and the server throws an error, then the client goes
    /// to lonewolf.
    Lonewolf,
}

pub fn get_conn_color(cs: ConnectionStatus) -> &'static str {
    match cs {
        ConnectionStatus::Offline => "#808080",
        ConnectionStatus::Connecting(color) => color,
        ConnectionStatus::LoggingIn => "#aa8800",
        ConnectionStatus::FailedToConnect => "#2255cc",
        ConnectionStatus::Connected => "#227722",
        ConnectionStatus::TwoTabs => "#444",
        ConnectionStatus::Lonewolf => "#2255cc",
        ConnectionStatus::LonewolfAndOffline => "#2255cc",
    }
}
