package z.zndroid.Tests

import rustlib.love_letter.LoveLetterSketch
import rustlib.love_letter.deserializeSketch
import rustlib.my_yrs_lib.BossOfYrs
import rustlib.my_yrs_lib.docFromSnapshot
import uniffi.protocol.Col
import uniffi.protocol.GetDataIn
import uniffi.protocol.SelectArgument
import uniffi.protocol.SelectArguments
import z.zndroid.DbManager
import z.zndroid.Storage.StorageAccess
import z.zndroid.db.reads.getPageBlob

object OutboxReplayCheck {
    const val ID = "outbox.replay"
    const val NAME = "Outbox replay (letters in order reproduce the stored blob)"

    private data class Letter(val id: Long, val sketch: LoveLetterSketch, val snapshot: ByteArray)

    suspend fun run(): DevCheckResult {
        if (DbManager.executeNative { }.isFailure) {
            return pass(ID, NAME, "Skipped (DB not ready)")
        }
        return try {
            val userId = StorageAccess.getUserId()
            val pagesOut = DbManager.getPages().getOrThrow()
            val pageIds = pagesOut.mapNotNull { (it.cols.getOrNull(1) as? Col.Text)?.v1 }

            val lettersOut = DbManager.getDataOrdered(uniffi.protocol.GetDataOrderedIn(
                "client_love_letter",
                SelectArguments.Single(SelectArgument.All),
                listOf("id", "snapshot_of_edit", "love_letter_sketch"),
                "id ASC"
            )).getOrThrow()

            val allLetters: List<Letter> = lettersOut.rows.mapNotNull { row ->
                val id = (row.cols.getOrNull(0) as? Col.Integer)?.v1 ?: return@mapNotNull null
                val snapshot = (row.cols.getOrNull(1) as? Col.Blob)?.v1 ?: return@mapNotNull null
                val sketchBytes = (row.cols.getOrNull(2) as? Col.Blob)?.v1 ?: return@mapNotNull null
                val sketch = try { deserializeSketch(sketchBytes) } catch (_: Exception) { return@mapNotNull null }
                Letter(id, sketch, snapshot)
            }

            val problems = mutableListOf<String>()

            for (pageId in pageIds) {
                val replayed = BossOfYrs(userId, System.currentTimeMillis().toString())
                val pageLetters = allLetters.filter { letter ->
                    when (val s = letter.sketch) {
                        is LoveLetterSketch.EditBlock -> s.targetPageId == pageId
                        is LoveLetterSketch.CreateNewBlock -> s.targetPageId == pageId
                        is LoveLetterSketch.CreateNewBlockWithId -> s.targetPageId == pageId
                        is LoveLetterSketch.RemoveBlock -> s.targetPageId == pageId
                        is LoveLetterSketch.MoveBlock -> s.targetPageId == pageId
                        is LoveLetterSketch.DisablePage -> false
                        is LoveLetterSketch.CreateNewPage -> false
                    }
                }

                try {
                    pageLetters.forEach { replayed.mergeWithSnapshot(it.snapshot) }
                    val replayedBlocks = replayed.getEntirePage()

                    val storedBlob = getPageBlob(pageId).getOrThrow()
                    val storedBoss = docFromSnapshot(storedBlob, userId, pageId, System.currentTimeMillis().toString())
                    val storedBlocks = try { storedBoss.getEntirePage() } finally { storedBoss.destroy() }

                    if (replayedBlocks.size != storedBlocks.size) {
                        problems.add("$pageId: replayed ${replayedBlocks.size} blocks, stored has ${storedBlocks.size}")
                    } else {
                        replayedBlocks.zip(storedBlocks).forEachIndexed { idx, (r, s) ->
                            if (r.idInYrs != s.idInYrs) {
                                problems.add("$pageId[${idx}]: id mismatch replayed=${r.idInYrs} stored=${s.idInYrs}")
                            } else if (r.text != s.text) {
                                problems.add("$pageId/${r.idInYrs}: text mismatch replayed='${r.text}' stored='${s.text}'")
                            }
                        }
                    }
                } finally {
                    replayed.destroy()
                }
            }

            if (problems.isEmpty()) {
                pass(ID, NAME, "Replayed letters for ${pageIds.size} pages; all match stored blobs. (Assumes letters aren't pruned.)")
            } else {
                fail(ID, NAME, "${problems.size} problem(s) across ${pageIds.size} pages.", problems.joinToString("\n"))
            }
        } catch (e: Exception) {
            fail(ID, NAME, "Exception: ${e.message}", e.stackTraceToString())
        }
    }
}
