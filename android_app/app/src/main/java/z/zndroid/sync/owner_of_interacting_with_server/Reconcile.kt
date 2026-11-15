package z.zndroid.sync.owner_of_interacting_with_server

import rustlib.love_letter.LoveLetter
import rustlib.love_letter.loveLetterToBytes
import rustlib.my_yrs_lib.docFromSnapshot
import rustlib.server_link.ReconcileOutcome
import rustlib.server_link.ServerLink
import rustlib.server_link.fingerprintOfBlob
import uniffi.protocol.GetDataIn
import uniffi.protocol.SelectArgument
import uniffi.protocol.SelectArguments
import z.zndroid.DbManager
import z.zndroid.Storage.StorageAccess
import z.zndroid.db.schema.PagesTableSchema

internal class LocalPage(val pageId: String, val blob: ByteArray, val isMainMenuPage: Boolean)

internal class MismatchedPage(val pageId: String, val clientBlob: ByteArray, val serverBlob: ByteArray)

internal class ClientOnlyPage(val pageId: String, val clientBlob: ByteArray, val isMainMenuPage: Boolean)

internal class ReconcileReady(
    val targetDriftVersion: String,
    val targetReconcileVersion: String,
    val mismatched: List<MismatchedPage>,
    val clientOnly: List<ClientOnlyPage>,
    val mountFingerprints: Map<String, String>
)

internal sealed class ReconcileSendResult {
    object Sent : ReconcileSendResult()
    object DriftMismatch : ReconcileSendResult()
    class Failed(val msg: String) : ReconcileSendResult()
}

/**
 * Mirrors web's load_and_classify: read local pages, fetch the server's
 * fingerprints and both versions, split local pages into mismatched and
 * client-only. Needs an open socket, not a Connected state (the asks are
 * exempt from the drift check). Throws on any failure.
 */
internal suspend fun loadAndClassify(link: ServerLink): ReconcileReady {
    val local = readLocalPages()
    val fingerprints = link.getPageFingerprints()
    val targetDriftVersion = fingerprints.driftVersion
    val serverFingerprintById = fingerprints.pages.associate { it.pageId to it.fingerprint }
    val targetReconcileVersion = link.getReconcileVersion()

    val mountFingerprints = local.associate { it.pageId to fingerprintOfBlob(it.blob) }

    val mismatchedIds = ArrayList<String>()
    val clientOnlyPages = ArrayList<ClientOnlyPage>()
    for (page in local) {
        val serverFingerprint = serverFingerprintById[page.pageId]
        if (serverFingerprint == null) {
            clientOnlyPages.add(ClientOnlyPage(page.pageId, page.blob, page.isMainMenuPage))
        } else if (mountFingerprints[page.pageId] != serverFingerprint) {
            mismatchedIds.add(page.pageId)
        }
    }

    val serverBlobById: Map<String, ByteArray> =
        if (mismatchedIds.isEmpty()) emptyMap()
        else link.getPagesByIds(mismatchedIds).associate { it.pageId to it.blobbedPage }

    val mismatched = ArrayList<MismatchedPage>()
    for (page in local) {
        if (page.pageId !in mismatchedIds) continue
        val serverBlob = serverBlobById[page.pageId] ?: continue
        mismatched.add(MismatchedPage(page.pageId, page.blob, serverBlob))
    }

    return ReconcileReady(
        targetDriftVersion = targetDriftVersion,
        targetReconcileVersion = targetReconcileVersion,
        mismatched = mismatched,
        clientOnly = clientOnlyPages,
        mountFingerprints = mountFingerprints
    )
}

/** Throws unless every local page is byte-for-byte what it was at mount. */
internal suspend fun verifyNothingChangedSinceMount(mountFingerprints: Map<String, String>) {
    val now = readLocalPages()
    if (now.size != mountFingerprints.size) {
        throw IllegalStateException("local page set changed since this page loaded; reload and start over")
    }
    for (page in now) {
        val mount = mountFingerprints[page.pageId]
        if (mount == null || mount != fingerprintOfBlob(page.blob)) {
            throw IllegalStateException(
                "local page ${page.pageId} changed since this page loaded; reload and start over"
            )
        }
    }
}

/** Encode the letters and send them as one batch. An empty batch sends nothing. */
internal suspend fun sendReconcileBatch(
    link: ServerLink,
    targetDriftVersion: String,
    targetReconcileVersion: String,
    letters: List<LoveLetter>
): ReconcileSendResult {
    if (letters.isEmpty()) return ReconcileSendResult.Sent
    val encoded = try {
        letters.map { loveLetterToBytes(it) }
    } catch (e: Exception) {
        return ReconcileSendResult.Failed("encode letter: ${e.message ?: e}")
    }
    return when (val outcome = link.sendReconcileLetters(targetDriftVersion, targetReconcileVersion, encoded)) {
        is ReconcileOutcome.Applied -> ReconcileSendResult.Sent
        is ReconcileOutcome.DriftVersionMismatch -> ReconcileSendResult.DriftMismatch
        is ReconcileOutcome.Other -> ReconcileSendResult.Failed(outcome.errMsg)
    }
}

/** Every local page, with its blob re-snapshotted through yrs exactly as web fingerprints it. */
private suspend fun readLocalPages(): List<LocalPage> {
    val userId = StorageAccess.getUserId()
    val time = System.currentTimeMillis().toString()
    val columns = listOf(
        PagesTableSchema.pageId.name,
        PagesTableSchema.blobbedPage.name,
        PagesTableSchema.isMainMenuPage.name
    )
    val out = DbManager.getData(
        GetDataIn(PagesTableSchema.TABLE_NAME, SelectArguments.Single(SelectArgument.All), columns)
    ).getOrThrow()
    val pages = ArrayList<LocalPage>()
    for (row in out.rows) {
        if (row.cols.size < 3) throw IllegalStateException("pages row has ${row.cols.size} cols, expected 3")
        val pageId = PagesTableSchema.pageId.unwrap(row.cols[0])
        val stored = PagesTableSchema.blobbedPage.unwrap(row.cols[1])
        val isMain = PagesTableSchema.isMainMenuPage.unwrap(row.cols[2]) == "true"
        val boss = docFromSnapshot(stored, userId, pageId, time)
        val blob = try {
            boss.snapshot()
        } finally {
            boss.destroy()
        }
        pages.add(LocalPage(pageId, blob, isMain))
    }
    return pages
}
