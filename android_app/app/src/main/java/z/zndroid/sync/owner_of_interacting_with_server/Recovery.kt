package z.zndroid.sync.owner_of_interacting_with_server

import kotlinx.coroutines.flow.update
import rustlib.love_letter.LoveLetter
import rustlib.server_link.ServerLink
import rustlib.server_link.setDriftVersion
import uniffi.protocol.Col
import uniffi.protocol.DeleteRowIn
import uniffi.protocol.GetDataIn
import uniffi.protocol.SelectArgument
import uniffi.protocol.SelectArguments
import z.zndroid.DbManager
import z.zndroid.WriteLock
import z.zndroid.db.schema.ClientLoveLetterTableSchema
import z.zndroid.db.schema.EveryBlockInExistenceTableSchema
import z.zndroid.db.schema.PagesTableSchema
import z.zndroid.db.schema.SeenServerLettersTableSchema

/**
 * Tables that hold synced data. key_value_storage and logs are kept.
 * Children first, parent last: every_block_in_existence and backlinks
 * have foreign keys pointing at pages.page_id, so pages must be deleted
 * after them or SQLite refuses with FOREIGN KEY constraint failed.
 */
private val LOCAL_DATA_TABLES = listOf(
    EveryBlockInExistenceTableSchema.TABLE_NAME,
    ClientLoveLetterTableSchema.TABLE_NAME,
    SeenServerLettersTableSchema.TABLE_NAME,
    PagesTableSchema.TABLE_NAME
)

/**
 * Reconcile where the server wins every conflict. Pages only this client
 * has are sent as CreateNewPage letters in one batch; then local data is
 * wiped and re-pulled. Local edits to mismatched pages are lost. Throws on failure.
 */
internal suspend fun finishReconcileServerWins(link: ServerLink, ready: ReconcileReady) {
    verifyNothingChangedSinceMount(ready.mountFingerprints)
    sendBatchOrThrow(link, ready, createPageLetters(ready.clientOnly))
    wipeLocalAndAdoptServer()
}

/**
 * Mirrors web's wipe_local_and_pull_from_server. Deletes local data and
 * forgets our drift version, then logs in again: afterLogin adopts the
 * server's version and startOverFreshGuard pulls every page.
 */
internal suspend fun wipeLocalAndAdoptServer() {
    WriteLock.withLock {
        DbManager.withTransaction {
            for (table in LOCAL_DATA_TABLES) wipeTable(table)
            writeKeyValue(CLIENT_DRIFT_KEY, "")
        }.getOrThrow()
    }
    SeenLetters.clearMemory()
    resetStartOverFresh()
    menuTickFlow.update { it + 1 }
    restartLogin()
}

/**
 * Wipe the server's dataset, adopt the drift version it enters, and push
 * every local page back. Needs an open socket (state Lonewolf).
 */
internal suspend fun wipeServerAndPushMine(link: ServerLink) {
    // Pause sync (debug entry can arrive here while Connected) so the send
    // and receive loops can't race the wipe.
    if (connState.value == ConnState.Connected) setConnState(ConnState.Lonewolf)
    // WipeDataset is not drift-exempt: stamp the server's current version first.
    setDriftVersion(link.getPageFingerprints().driftVersion)
    val info = link.wipeDataset()

    try {
        val ready = loadAndClassify(link)
        sendBatchOrThrow(link, ready, createPageLetters(ready.clientOnly))
    } catch (e: Exception) {
        throw IllegalStateException(
            "the server was wiped but pushing this device's pages failed: ${e.message ?: e}. " +
                "Run \"Wipe server\" again; do not use Reconcile.",
            e
        )
    }

    writeKeyValue(CLIENT_DRIFT_KEY, info.driftVersion)
    setConnState(ConnState.LoggingIn)
    afterLogin(info)
}

private suspend fun sendBatchOrThrow(link: ServerLink, ready: ReconcileReady, letters: List<LoveLetter>) {
    when (val r = sendReconcileBatch(link, ready.targetDriftVersion, ready.targetReconcileVersion, letters)) {
        ReconcileSendResult.Sent -> Unit
        ReconcileSendResult.DriftMismatch ->
            throw IllegalStateException("server dataset changed during reconcile; reload and start over")
        is ReconcileSendResult.Failed ->
            throw IllegalStateException("reconcile batch failed: ${r.msg}")
    }
}

private fun createPageLetters(pages: List<ClientOnlyPage>): List<LoveLetter> =
    pages.map {
        LoveLetter.CreateNewPage(
            pageId = it.pageId,
            isMainMenuPage = it.isMainMenuPage,
            snapshotOfEdit = it.clientBlob
        )
    }

/** Must run inside a transaction. */
private suspend fun wipeTable(table: String) {
    val rows = DbManager.getData(
        GetDataIn(table, SelectArguments.Single(SelectArgument.All), listOf("id"))
    ).getOrThrow().rows
    for (row in rows) {
        val id = (row.cols.firstOrNull() as? Col.Integer)?.v1?.toString() ?: continue
        DbManager.deleteRow(DeleteRowIn(table, id)).getOrThrow()
    }
}

/**
 * Socket open (Lonewolf): go to LoggingIn and let the receive loop log in.
 * Socket closed: go to Offline; the reconnect loop does the rest.
 */
private fun restartLogin() {
    when (connState.value) {
        ConnState.Lonewolf, ConnState.Connected -> {
            setConnState(ConnState.LoggingIn)
            SpontaneousInbox.socketReconnected.trySend(Unit)
        }
        ConnState.LonewolfAndOffline -> setConnState(ConnState.Offline)
        else -> Unit
    }
}
