package z.zndroid.sync.owner_of_interacting_with_server

import android.util.Log
import kotlinx.coroutines.launch
import rustlib.love_letter.buildALoveLetter
import rustlib.love_letter.deserializeSketch
import rustlib.love_letter.loveLetterToBytes
import rustlib.server_link.LinkException
import rustlib.server_link.ServerLink
import uniffi.protocol.DeleteRowIn
import uniffi.protocol.GetDataOrderedIn
import uniffi.protocol.SelectArgument
import uniffi.protocol.SelectArguments
import z.zndroid.DbManager
import z.zndroid.actions.sendWakeup
import z.zndroid.components.GlobalPopupManager
import z.zndroid.db.schema.ClientLoveLetterTableSchema

private const val TAG = "SyncSend"

private class PendingRow(
    val id: Long,
    val snapshotOfEdit: ByteArray,
    val loveLetterSketch: ByteArray
)

private sealed class SendResult {
    object Ok : SendResult()
    object Transport : SendResult()
    class Failed(val msg: String) : SendResult()
}

/**
 * client_love_letter is the queue. Each wakeup drains it oldest first:
 * send, delete the row on ok. A transport failure or a closed gate
 * leaves the row and stops; the next kick (user action or reconnect)
 * tries again. After 3 consecutive failures on the same letter, popup once.
 */
internal fun spawnSender(link: ServerLink) {
    syncScope.launch {
        var currentLetterId: Long? = null
        var consecutiveFailures = 0
        var poppedThisStreak = false

        for (wakeup in sendWakeup) {
            while (true) {
                val row: PendingRow? = try {
                    nextPendingRow()
                } catch (e: Exception) {
                    Log.e(TAG, "read outbox failed", e)
                    null
                }
                if (row == null) break

                if (currentLetterId != row.id) {
                    currentLetterId = row.id
                    consecutiveFailures = 0
                    poppedThisStreak = false
                }

                if (!syncMayRun()) break

                when (val result = sendOne(link, row)) {
                    is SendResult.Ok -> {
                        try {
                            DbManager.deleteRow(
                                DeleteRowIn(ClientLoveLetterTableSchema.TABLE_NAME, row.id.toString())
                            ).getOrThrow()
                        } catch (e: Exception) {
                            Log.e(TAG, "delete outbox row ${row.id} failed", e)
                            break
                        }
                        consecutiveFailures = 0
                        poppedThisStreak = false
                    }
                    is SendResult.Transport -> break
                    is SendResult.Failed -> {
                        consecutiveFailures += 1
                        Log.w(TAG, "letter ${row.id}: ${result.msg}")
                        if (consecutiveFailures >= 3 && !poppedThisStreak) {
                            poppedThisStreak = true
                            GlobalPopupManager.showError(
                                "Server rejected a change (letter ${row.id}): ${result.msg}. Will keep retrying on the next edit or reconnect."
                            )
                        }
                        break
                    }
                }
            }
        }
    }
}

private suspend fun nextPendingRow(): PendingRow? {
    val columns = listOf(
        ClientLoveLetterTableSchema.id.name,
        ClientLoveLetterTableSchema.snapshotOfEdit.name,
        ClientLoveLetterTableSchema.loveLetterSketch.name
    )
    val out = DbManager.getDataOrdered(
        GetDataOrderedIn(
            ClientLoveLetterTableSchema.TABLE_NAME,
            SelectArguments.Single(SelectArgument.All),
            columns,
            "id ASC"
        )
    ).getOrThrow()
    val row = out.rows.firstOrNull() ?: return null
    if (row.cols.size < 3) {
        throw IllegalStateException("client_love_letter row has ${row.cols.size} cols, expected 3")
    }
    return PendingRow(
        id = ClientLoveLetterTableSchema.id.unwrap(row.cols[0]),
        snapshotOfEdit = ClientLoveLetterTableSchema.snapshotOfEdit.unwrap(row.cols[1]),
        loveLetterSketch = ClientLoveLetterTableSchema.loveLetterSketch.unwrap(row.cols[2])
    )
}

private suspend fun sendOne(link: ServerLink, row: PendingRow): SendResult {
    val bytes: ByteArray = try {
        val sketch = deserializeSketch(row.loveLetterSketch)
        val letter = buildALoveLetter(sketch, row.snapshotOfEdit)
        loveLetterToBytes(letter)
    } catch (e: Exception) {
        return SendResult.Failed("build letter ${row.id}: ${e.message ?: e}")
    }
    return try {
        link.sendLoveLetter(bytes)
        SendResult.Ok
    } catch (e: LinkException.Transport) {
        SendResult.Transport
    } catch (e: LinkException) {
        SendResult.Failed(e.message ?: e.toString())
    }
}
