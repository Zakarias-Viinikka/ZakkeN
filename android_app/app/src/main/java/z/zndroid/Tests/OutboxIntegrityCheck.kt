package z.zndroid.Tests

import rustlib.love_letter.LoveLetterSketch
import rustlib.love_letter.deserializeSketch
import uniffi.protocol.Col
import uniffi.protocol.GetDataIn
import uniffi.protocol.SelectArgument
import uniffi.protocol.SelectArguments
import z.zndroid.DbManager

object OutboxIntegrityCheck {
    const val ID = "outbox.integrity"
    const val NAME = "Outbox integrity (sketches decode, targets exist)"

    suspend fun run(): DevCheckResult {
        if (DbManager.executeNative { }.isFailure) {
            return pass(ID, NAME, "Skipped (DB not ready)")
        }
        return try {
            val pagesOut = DbManager.getPages().getOrThrow()
            val existingPages = pagesOut.mapNotNull { (it.cols.getOrNull(1) as? Col.Text)?.v1 }.toSet()

            val letters = DbManager.getData(GetDataIn(
                "client_love_letter",
                SelectArguments.Single(SelectArgument.All),
                listOf("snapshot_of_edit", "love_letter_sketch", "session_id", "target_id")
            )).getOrThrow()

            val problems = mutableListOf<String>()
            letters.rows.forEachIndexed { index, row ->
                val sketchBytes = (row.cols.getOrNull(1) as? Col.Blob)?.v1
                if (sketchBytes == null) {
                    problems.add("row #$index: love_letter_sketch column missing or not Blob")
                    return@forEachIndexed
                }
                val sketch = try {
                    deserializeSketch(sketchBytes)
                } catch (e: Exception) {
                    problems.add("row #$index: deserializeSketch failed: ${e.message}")
                    return@forEachIndexed
                }
                val targetPageId: String = when (sketch) {
                    is LoveLetterSketch.EditBlock -> sketch.targetPageId
                    is LoveLetterSketch.CreateNewBlock -> sketch.targetPageId
                    is LoveLetterSketch.CreateNewBlockWithId -> sketch.targetPageId
                    is LoveLetterSketch.RemoveBlock -> sketch.targetPageId
                    is LoveLetterSketch.MoveBlock -> sketch.targetPageId
                    is LoveLetterSketch.DisablePage -> sketch.pageId
                    is LoveLetterSketch.CreateNewPage -> sketch.pageId
                }
                val isCreateNewPage = sketch is LoveLetterSketch.CreateNewPage
                if (!existingPages.contains(targetPageId) && !isCreateNewPage) {
                    problems.add("row #$index: sketch target page $targetPageId not found in pages table")
                }
            }

            if (problems.isEmpty()) {
                pass(ID, NAME, "Decoded ${letters.rows.size} letters, all targets resolve.")
            } else {
                fail(ID, NAME, "${problems.size} problem(s) across ${letters.rows.size} letters.", problems.joinToString("\n"))
            }
        } catch (e: Exception) {
            fail(ID, NAME, "Exception: ${e.message}", e.stackTraceToString())
        }
    }
}
