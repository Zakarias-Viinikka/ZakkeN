package z.zndroid.sync.owner_of_interacting_with_server

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow

/** Mirrors web's ConnState. */
enum class ConnState {
    Offline,
    Connecting,
    LoggingIn,
    Connected,
    Lonewolf,
    LonewolfAndOffline,
    FailedToConnect,
}

private val connStateFlow = MutableStateFlow(ConnState.Offline)

val connState: StateFlow<ConnState> = connStateFlow

internal fun setConnState(next: ConnState) {
    connStateFlow.value = next
}

/** True only when sync is allowed to run. Reads the state, nothing else. */
fun syncMayRun(): Boolean = connStateFlow.value == ConnState.Connected
