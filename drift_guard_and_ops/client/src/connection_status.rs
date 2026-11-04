#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConnectionStatus {
    Offline,
    Connecting,
    /// Either the in-between step between connecting and failed to connect due to
    /// failing to connect in general. But it's also the state for when the client
    /// tries to log in with x username and it fails for whatever reason.
    FailedToConnect,
    Connected,
    LoneWolfAndOffline,
    /// Means connected to the server, but sync is disabled for whatever reason.
    /// Either because the user has been forced into lonewolf due to another user
    /// forcing the server to a new dataset. Or it happens because the client
    /// deliberately unsynced. It's also the state the client ends up in due to
    /// specific error handling — for example, if the client tries to push a love
    /// letter to the server and the server throws an error, then the client goes
    /// to lonewolf.
    Lonewolf,
}