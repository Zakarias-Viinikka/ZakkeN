package z.zndroid.sync.owner_of_interacting_with_server

import android.util.Log
import rustlib.server_link.IncomingLetter
import rustlib.server_link.LinkListener
import rustlib.server_link.LinkStatus

/**
 * Called from a Rust thread. Must return fast: set state or trySend, nothing else.
 * The `when` has no else branch on purpose, so a new LinkStatus variant
 * stops the build here.
 */
internal object SyncListener : LinkListener {
    private const val TAG = "SyncListener"

    override fun onConnStatus(status: LinkStatus) {
        when (status) {
            LinkStatus.CONNECTING -> setConnState(ConnState.Connecting)
            LinkStatus.CONNECTED -> {
                setConnState(ConnState.LoggingIn)
                SpontaneousInbox.socketReconnected.trySend(Unit)
            }
            LinkStatus.FAILED -> setConnState(ConnState.Offline)
        }
    }

    override fun onLoveLetter(letter: IncomingLetter) {
        val sent = SpontaneousInbox.letters.trySend(letter)
        if (sent.isFailure) {
            Log.e(TAG, "onLoveLetter: could not queue letter ${letter.letterId}: ${sent.exceptionOrNull()}")
        }
    }

    override fun onDriftChanged(driftVersion: String, driftReason: String) {
        val sent = SpontaneousInbox.driftChanged.trySend(DriftChanged(driftVersion, driftReason))
        if (sent.isFailure) {
            Log.e(TAG, "onDriftChanged: could not queue: ${sent.exceptionOrNull()}")
        }
    }
}
