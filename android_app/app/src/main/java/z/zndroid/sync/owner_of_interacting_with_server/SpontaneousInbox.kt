package z.zndroid.sync.owner_of_interacting_with_server

import kotlinx.coroutines.channels.Channel
import rustlib.server_link.IncomingLetter

data class DriftChanged(val driftVersion: String, val driftReason: String)

/**
 * Where the listener drops everything the server pushes. The receive
 * loop (next steps) is the only reader.
 */
internal object SpontaneousInbox {
    /** One item per love letter the server pushed. Nothing may be lost. */
    val letters = Channel<IncomingLetter>(Channel.UNLIMITED)

    /** "The socket is open, log in again." Conflated: one pending signal is enough. */
    val socketReconnected = Channel<Unit>(Channel.CONFLATED)

    /** "Login finished, pull what the server still holds." Conflated. */
    val loggedIn = Channel<Unit>(Channel.CONFLATED)

    /** The server said our dataset is no longer current. */
    val driftChanged = Channel<DriftChanged>(Channel.UNLIMITED)
}
