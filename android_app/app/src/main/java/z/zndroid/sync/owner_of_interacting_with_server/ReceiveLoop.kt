package z.zndroid.sync.owner_of_interacting_with_server

import android.util.Log
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.selects.select
import rustlib.love_letter.deserializeLoveLetter
import rustlib.love_letter.LoveLetter
import rustlib.server_link.IncomingLetter
import rustlib.server_link.ServerLink
import z.zndroid.components.GlobalPopupManager

private const val TAG = "SyncReceive"

internal val menuTickFlow = MutableStateFlow(0)

/** Bumps every time a server letter changed local state. The menu observes this. */
val menuTick: StateFlow<Int> = menuTickFlow

/**
 * One task. Handles pushed letters, and after every successful login
 * pulls whatever the server still holds for us.
 */
internal fun spawnReceiver(link: ServerLink) {
    syncScope.launch {
        try {
            SeenLetters.load()
        } catch (e: Exception) {
            Log.w(TAG, "SeenLetters.load failed", e)
        }
        while (true) {
            val keepGoing = select<Boolean> {
                SpontaneousInbox.letters.onReceiveCatching { result ->
                    val item = result.getOrNull()
                    if (item == null) false else handleOne(link, item)
                }
                SpontaneousInbox.socketReconnected.onReceiveCatching { result ->
                    if (result.isClosed) false else loginAndPull(link)
                }
            }
            if (!keepGoing) break
        }
        Log.w(TAG, "receive loop exiting")
    }
}

private suspend fun loginAndPull(link: ServerLink): Boolean {
    if (!loginAfterReconnect(link)) return true
    return pullAndHandle(link)
}

private suspend fun pullAndHandle(link: ServerLink): Boolean {
    if (!syncMayRun()) return true
    try {
        startOverFreshGuard(link)
    } catch (e: Exception) {
        Log.e(TAG, "start over fresh failed", e)
        GlobalPopupManager.showError("start over fresh failed: ${e.message ?: e}")
    }
    val letters = try {
        link.pullUnsentLetters()
    } catch (e: Exception) {
        Log.w(TAG, "pull failed", e)
        return true
    }
    return handleBatch(link, letters)
}

private suspend fun handleBatch(link: ServerLink, letters: List<IncomingLetter>): Boolean {
    for (item in letters) {
        if (!handleOne(link, item)) return false
    }
    return true
}

/** Returns false only when the loop must stop (fatal data problem). */
private suspend fun handleOne(link: ServerLink, item: IncomingLetter): Boolean {
    if (!syncMayRun()) return true
    if (!letterIsOnOurDriftVersion(item)) return true

    if (!SeenLetters.claim(item.letterId)) {
        sendAck(link, item.letterId)
        return true
    }

    val letter: LoveLetter = try {
        deserializeLoveLetter(item.loveLetter)
    } catch (e: Exception) {
        Log.e(TAG, "decode letter ${item.letterId}", e)
        SeenLetters.release(item.letterId)
        return true
    }
    val createdNewPage = letter is LoveLetter.CreateNewPage

    val result = try {
        applyLetter(letter)
    } catch (e: Exception) {
        Log.e(TAG, "apply letter ${item.letterId} failed", e)
        SeenLetters.release(item.letterId)
        GlobalPopupManager.showError(
            "Data problem while receiving an edit. Copy anything you're afraid of losing, then restart the app."
        )
        return false
    }

    return when (result) {
        is ApplyResult.Applied -> {
            try {
                SeenLetters.persist(item.letterId)
            } catch (e: Exception) {
                Log.w(TAG, "persist seen ${item.letterId} failed", e)
            }
            sendAck(link, item.letterId)
            menuTickFlow.update { it + 1 }
            if (createdNewPage) {
                // Letters that targeted this page earlier were left unacked; ask again.
                val more = try {
                    link.pullUnsentLetters()
                } catch (e: Exception) {
                    Log.w(TAG, "re-pull after new page failed", e)
                    emptyList<IncomingLetter>()
                }
                handleBatch(link, more)
            } else {
                true
            }
        }
        is ApplyResult.UnknownPage -> recoverUnknownPage(link, item, letter, result.pageId)
        is ApplyResult.Gated -> {
            SeenLetters.release(item.letterId)
            true
        }
    }
}

/**
 * The letter targets a page we don't have. Fetch it from the server,
 * apply it, then apply the letter. If the server doesn't have the page
 * either, the letter can never apply: ack it so it stops replaying.
 * Returns false only when the loop must stop (fatal data problem).
 */
private suspend fun recoverUnknownPage(
    link: ServerLink,
    item: IncomingLetter,
    letter: LoveLetter,
    pageId: String
): Boolean {
    val fetched = try {
        link.getPagesByIds(listOf(pageId)).firstOrNull()
    } catch (e: Exception) {
        Log.w(TAG, "letter ${item.letterId}: fetching page $pageId failed", e)
        SeenLetters.release(item.letterId)
        return true
    }
    if (fetched == null) {
        Log.w(TAG, "letter ${item.letterId} targets page $pageId which the server doesn't have; acking and dropping")
        try {
            SeenLetters.persist(item.letterId)
        } catch (e: Exception) {
            Log.w(TAG, "persist seen ${item.letterId} failed", e)
        }
        sendAck(link, item.letterId)
        return true
    }

    val again = try {
        applyBootstrapPage(fetched)
        applyLetter(letter)
    } catch (e: Exception) {
        SeenLetters.release(item.letterId)
        if (!syncMayRun()) return true
        Log.e(TAG, "recover page $pageId for letter ${item.letterId} failed", e)
        GlobalPopupManager.showError(
            "Data problem while receiving an edit. Copy anything you're afraid of losing, then restart the app."
        )
        return false
    }

    return when (again) {
        is ApplyResult.Applied -> {
            try {
                SeenLetters.persist(item.letterId)
            } catch (e: Exception) {
                Log.w(TAG, "persist seen ${item.letterId} failed", e)
            }
            sendAck(link, item.letterId)
            menuTickFlow.update { it + 1 }
            true
        }
        is ApplyResult.UnknownPage, is ApplyResult.Gated -> {
            Log.w(TAG, "letter ${item.letterId} still not applied after fetching page $pageId")
            SeenLetters.release(item.letterId)
            true
        }
    }
}

private suspend fun letterIsOnOurDriftVersion(item: IncomingLetter): Boolean {
    val ours = try {
        readKeyValue(CLIENT_DRIFT_KEY)
    } catch (e: Exception) {
        Log.e(TAG, "couldn't read our drift version", e)
        return false
    }
    val accepts = ours == item.driftVersion
    if (!accepts) {
        Log.w(TAG, "letter ${item.letterId} arrived on drift version ${item.driftVersion} but we're on $ours - dropping")
    }
    return accepts
}

private fun sendAck(link: ServerLink, letterId: Long) {
    syncScope.launch {
        try {
            link.ack(letterId)
        } catch (e: Exception) {
            Log.w(TAG, "ack $letterId failed", e)
        }
    }
}
