package z.zndroid.Tests

import android.util.Log
import rustlib.love_letter.LoveLetterSketch
import rustlib.love_letter.deserializeSketch
import rustlib.my_yrs_lib.docFromSnapshot
import uniffi.protocol.Col
import uniffi.protocol.GetDataIn
import uniffi.protocol.GetDataOrderedIn
import uniffi.protocol.SelectArgument
import uniffi.protocol.SelectArguments
import z.zndroid.DbManager
import z.zndroid.Storage.StorageAccess
import z.zndroid.db.reads.getPageBlob

object OutboxReplayCheck {
    const val ID = "outbox.replay"
    const val NAME = "Outbox replay (letters in order reproduce the stored blob)"
    private const val TAG = "OutboxReplay"

    private data class Letter(val id: Long, val sketch: LoveLetterSketch, val snapshot: ByteArray)

    private fun shortIds(blocks: List<rustlib.my_yrs_lib.Block>): String =
        blocks.take(8).joinToString(", ") { "${it.idInYrs.take(8)}='${it.text.take(24)}'" }

    suspend fun run(): DevCheckResult {
        try {
            DbManager.awaitReady()
        } catch (e: Exception) {
            return fail(ID, NAME, "DB not ready: ${e.message}", e.stackTraceToString())
        }
        return try {
            val userId = StorageAccess.getUserId()
            val pageIds = DbManager.getData(GetDataIn(
                "pages",
                SelectArguments.Single(SelectArgument.All),
                listOf("page_id")
            )).getOrThrow().rows.mapNotNull {
                (it.cols.firstOrNull() as? Col.Text)?.v1
            }

            val lettersOut = DbManager.getDataOrdered(GetDataOrderedIn(
                "client_love_letter",
                SelectArguments.Single(SelectArgument.All),
                listOf("id", "snapshot_of_edit", "love_letter_sketch"),
                "id ASC"
            )).getOrThrow()

            if (lettersOut.rows.isEmpty()) {
                Log.i(TAG, "outbox empty (0 rows across ${pageIds.size} pages); skipping")
                return skip(ID, NAME, "Outbox is empty (0 letters in client_love_letter); nothing to replay.")
            }

            var decodeFailures = 0
            var nullCols = 0
            val allLetters: List<Letter> = lettersOut.rows.mapNotNull { row ->
                val id = (row.cols.getOrNull(0) as? Col.Integer)?.v1
                val snapshot = (row.cols.getOrNull(1) as? Col.Blob)?.v1
                val sketchBytes = (row.cols.getOrNull(2) as? Col.Blob)?.v1
                if (id == null || snapshot == null || sketchBytes == null) {
                    nullCols++
                    return@mapNotNull null
                }
                val sketch = try {
                    deserializeSketch(sketchBytes)
                } catch (e: Exception) {
                    decodeFailures++
                    Log.w(TAG, "row id=$id deserializeSketch failed: ${e.message}")
                    return@mapNotNull null
                }
                Letter(id, sketch, snapshot)
            }

            Log.i(
                TAG,
                "loaded ${allLetters.size} letters from ${lettersOut.rows.size} rows; " +
                    "decodeFailures=$decodeFailures nullCols=$nullCols pages=${pageIds.size}"
            )

            val problems = mutableListOf<String>()
            if (decodeFailures > 0 || nullCols > 0) {
                problems.add("Outbox contains $decodeFailures decode failure(s) and $nullCols missing/invalid column(s).")
            }

            var replayedCount = 0
            var prunedCount = 0

            for (pageId in pageIds) {
                val pageLetters = allLetters.filter { letter ->
                    when (val s = letter.sketch) {
                        is LoveLetterSketch.CreateNewPage -> s.pageId == pageId
                        is LoveLetterSketch.EditBlock -> s.targetPageId == pageId
                        is LoveLetterSketch.CreateNewBlock -> s.targetPageId == pageId
                        is LoveLetterSketch.CreateNewBlockWithId -> s.targetPageId == pageId
                        is LoveLetterSketch.RemoveBlock -> s.targetPageId == pageId
                        is LoveLetterSketch.MoveBlock -> s.targetPageId == pageId
                        is LoveLetterSketch.DisablePage -> false
                    }
                }

                val genesisIdx = pageLetters.indexOfFirst { it.sketch is LoveLetterSketch.CreateNewPage }
                if (genesisIdx == -1) {
                    prunedCount++
                    Log.i(TAG, "page $pageId: no CreateNewPage in outbox (${pageLetters.size} later letter(s)); pruned")
                    continue
                }
                if (genesisIdx > 0) {
                    val p = "$pageId: found $genesisIdx letter(s) before CreateNewPage"
                    problems.add(p)
                    Log.w(TAG, p)
                }

                replayedCount++
                val genesis = pageLetters[genesisIdx]
                val replayed = docFromSnapshot(genesis.snapshot, userId, pageId, System.currentTimeMillis().toString())
                try {
                    pageLetters.drop(genesisIdx + 1).forEach { replayed.mergeWithSnapshot(it.snapshot) }
                    val replayedBlocks = replayed.getEntirePage()

                    val storedBlob = getPageBlob(pageId).getOrThrow()
                    val storedBoss = docFromSnapshot(storedBlob, userId, pageId, System.currentTimeMillis().toString())
                    val storedBlocks = try { storedBoss.getEntirePage() } finally { storedBoss.destroy() }

                    Log.i(
                        TAG,
                        "page $pageId: letters=${pageLetters.size} replayed=${replayedBlocks.size} stored=${storedBlocks.size}"
                    )
                    if (replayedBlocks.size != storedBlocks.size) {
                        Log.w(TAG, "page $pageId replayed: ${shortIds(replayedBlocks)}")
                        Log.w(TAG, "page $pageId stored:   ${shortIds(storedBlocks)}")
                    }

                    if (replayedBlocks.size != storedBlocks.size) {
                        problems.add("$pageId: replayed ${replayedBlocks.size} blocks, stored has ${storedBlocks.size}")
                    } else {
                        replayedBlocks.zip(storedBlocks).forEachIndexed { idx, (r, s) ->
                            if (r.idInYrs != s.idInYrs) {
                                val p = "$pageId[${idx}]: id mismatch replayed=${r.idInYrs} stored=${s.idInYrs}"
                                problems.add(p)
                                Log.w(TAG, p)
                            } else if (r.text != s.text) {
                                val p = "$pageId/${r.idInYrs}: text mismatch replayed='${r.text}' stored='${s.text}'"
                                problems.add(p)
                                Log.w(TAG, p)
                            }
                        }
                    }
                } finally {
                    replayed.destroy()
                }
            }

            Log.i(TAG, "done: ${problems.size} problem(s) across $replayedCount replayable page(s), $prunedCount pruned")

            if (problems.isNotEmpty()) {
                fail(
                    ID,
                    NAME,
                    "${problems.size} problem(s) across $replayedCount replayable page(s) ($prunedCount pruned).",
                    problems.joinToString("\n")
                )
            } else if (replayedCount == 0) {
                skip(
                    ID,
                    NAME,
                    "0 replayable pages ($prunedCount/${pageIds.size} had genesis already sent/pruned; ${allLetters.size} outbox letter(s))."
                )
            } else {
                pass(
                    ID,
                    NAME,
                    "Replayed $replayedCount page(s) from retained genesis ($prunedCount pruned); all match stored blobs."
                )
            }
        } catch (e: Exception) {
            fail(ID, NAME, "Exception: ${e.message}", e.stackTraceToString())
        }
    }
}