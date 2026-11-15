package z.zndroid.actions

import kotlinx.coroutines.channels.Channel

/**
 * Wakeup line for the send loop. Conflated: any number of kicks while the
 * loop is busy collapse into one pending wakeup, which is all it needs
 * because it drains client_love_letter until empty.
 */
internal val sendWakeup = Channel<Unit>(Channel.CONFLATED)

/**
 * Called after every committed write. Never suspends, never throws.
 */
fun kickSend() {
    sendWakeup.trySend(Unit)
}