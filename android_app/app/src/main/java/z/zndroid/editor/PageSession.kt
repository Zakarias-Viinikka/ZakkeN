package z.zndroid.editor

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.input.TextFieldValue
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.withContext
import rustlib.my_yrs_lib.BossOfYrs
import rustlib.my_yrs_lib.PositionToInsert
import rustlib.text_diff.DiffResult
import rustlib.text_diff.getDiff
import z.zndroid.db.reads.BlockMirrorRow
import java.util.UUID

class BlockRow(
    val blockId: String,
    val isTitle: Boolean,
    initialText: String
) {
    var textFieldValue by mutableStateOf(TextFieldValue(initialText))

    /** Bumped when the user types the first character into an empty body block. EditableBlock plays the indent transition on change. */
    var indentGen by mutableIntStateOf(0)

    /** Last indentGen already animated (plain field: stops a re-animation when the row scrolls back into view). */
    var indentPlayedGen = 0
    val focusRequester = FocusRequester()
}

data class PendingFocus(val blockId: String, val offset: Int)

/**
 * A just-deleted block kept only for its drop animation. It is NOT part of
 * PageSession.rows (the DB delete and all index logic already treat it as gone).
 * anchorBlockId is the block that was directly above it, so the UI can draw the ghost
 * in place.
 */
class GhostRow(val blockId: String, val text: String, val anchorBlockId: String)

class PageSession(
    val boss: BossOfYrs,
    val pageId: String,
    val isMainMenu: Boolean,
    mirrorRows: List<BlockMirrorRow>,
    private val onFailure: () -> Unit
) {
    val rows = mutableStateListOf<BlockRow>()
    val pendingFocus = mutableStateOf<PendingFocus?>(null)
    val ghosts = mutableStateListOf<GhostRow>()
    private val batcher = PageBatcher(boss, pageId, isMainMenu, onFailure)

    init {
        for (row in mirrorRows) {
            rows.add(BlockRow(row.blockId, row.isTitle, row.content))
        }
    }

    private fun spawnGhost(row: BlockRow, anchorBlockId: String) {
        ghosts.add(GhostRow(row.blockId, row.textFieldValue.text, anchorBlockId))
    }

    fun removeGhost(ghost: GhostRow) {
        ghosts.remove(ghost)
    }

    fun onInput(rowIndex: Int, newValue: TextFieldValue) {
        val row = rows[rowIndex]
        val old = row.textFieldValue
        if (newValue.text == old.text) {
            row.textFieldValue = newValue
            return
        }
        val diff = computeDiff(old.text, newValue.text)
        // First input into an empty body block, and that input is >10 chars:
        // indent transition. Fires on a >10-char paste or IME word commit;
        // typing one character at a time never reaches this (each later
        // keystroke's `old` is non-empty).
        if (!row.isTitle && old.text.isEmpty() && newValue.text.length > 10) {
            row.indentGen++
        }
        batcher.pushEdit(row.blockId, diff)
        row.textFieldValue = newValue
    }

    fun onEnter(rowIndex: Int, cursorPos: Int) {
        val row = rows[rowIndex]
        val text = row.textFieldValue.text
        val cursor = cursorPos.coerceIn(0, text.length)
        val head = text.substring(0, cursor)
        val tail = text.substring(cursor)

        if (head != text) {
            batcher.pushEdit(row.blockId, computeDiff(text, head))
        }

        val newId = UUID.randomUUID().toString().replace("-", "")
        val above = row.blockId
        val below = rows.getOrNull(rowIndex + 1)?.blockId
        batcher.pushAdd(newId, tail, PositionToInsert.Between(above, below))

        val newRow = BlockRow(newId, false, tail)
        rows.add(rowIndex + 1, newRow)
        row.textFieldValue = row.textFieldValue.copy(
            text = head,
            selection = TextRange(head.length)
        )
        pendingFocus.value = PendingFocus(newId, 0)
    }

    /**
     * Handles input containing one or more newlines (soft-keyboard Enter or
     * multi-line paste). Splits normalized lines into the current block plus
     * newly inserted blocks below it, preserving text before/after the paste
     * and placing focus at the exact cursor location in the target block.
     */
    fun onMultiLineInput(rowIndex: Int, newValue: TextFieldValue) {
        val row = rows[rowIndex]
        val oldText = row.textFieldValue.text
        val rawText = newValue.text
        val normalized = rawText.replace("\r\n", "\n").replace('\r', '\n')
        val parts = normalized.split('\n')
        if (parts.size <= 1) {
            onInput(rowIndex, newValue.copy(text = normalized))
            return
        }

        val rawCursor = newValue.selection.start.coerceIn(0, rawText.length)
        val normalizedCursor = rawText.substring(0, rawCursor)
            .replace("\r\n", "\n")
            .replace('\r', '\n')
            .length

        var remaining = normalizedCursor
        var targetPartIdx = parts.lastIndex
        var targetOffset = parts.last().length
        for (i in parts.indices) {
            val lineLen = parts[i].length
            if (remaining <= lineLen) {
                targetPartIdx = i
                targetOffset = remaining
                break
            }
            remaining -= lineLen + 1
        }

        val firstText = parts[0]
        if (firstText != oldText) {
            batcher.pushEdit(row.blockId, computeDiff(oldText, firstText))
        }
        row.textFieldValue = row.textFieldValue.copy(
            text = firstText,
            selection = TextRange(firstText.length)
        )

        val originalBelow = rows.getOrNull(rowIndex + 1)?.blockId
        var prevId = row.blockId
        val insertedRows = ArrayList<BlockRow>(parts.size - 1)
        for (i in 1 until parts.size) {
            val newId = UUID.randomUUID().toString().replace("-", "")
            val partText = parts[i]
            batcher.pushAdd(newId, partText, PositionToInsert.Between(prevId, originalBelow))
            val newRow = BlockRow(newId, false, partText)
            rows.add(rowIndex + i, newRow)
            insertedRows.add(newRow)
            prevId = newId
        }

        val targetRow = if (targetPartIdx == 0) row else insertedRows[targetPartIdx - 1]
        if (targetPartIdx == 0) {
            targetRow.textFieldValue = targetRow.textFieldValue.copy(
                selection = TextRange(targetOffset)
            )
        }
        pendingFocus.value = PendingFocus(targetRow.blockId, targetOffset)
    }

    fun onBackspaceAtStart(rowIndex: Int) {
        val row = rows[rowIndex]
        if (row.isTitle) return
        val aboveIndex = rowIndex - 1
        if (aboveIndex < 0) return
        val above = rows[aboveIndex]
        if (above.isTitle) return

        if (row.textFieldValue.text.isEmpty()) {
            val targetOffset = above.textFieldValue.text.length
            above.textFieldValue = above.textFieldValue.copy(
                selection = TextRange(targetOffset)
            )
            runCatching { above.focusRequester.requestFocus() }
            batcher.pushDelete(row.blockId)
            rows.removeAt(rowIndex)
            spawnGhost(row, above.blockId)
            pendingFocus.value = PendingFocus(above.blockId, targetOffset)
        } else if (above.textFieldValue.text.isEmpty()) {
            batcher.pushDelete(above.blockId)
            rows.removeAt(aboveIndex)
            spawnGhost(above, rows[aboveIndex - 1].blockId)
            pendingFocus.value = PendingFocus(row.blockId, 0)
        } else {
            val joinOffset = above.textFieldValue.text.length
            val combined = above.textFieldValue.text + row.textFieldValue.text
            above.textFieldValue = above.textFieldValue.copy(
                text = combined,
                selection = TextRange(joinOffset)
            )
            runCatching { above.focusRequester.requestFocus() }
            batcher.pushEdit(above.blockId, computeDiff(above.textFieldValue.text, combined))
            batcher.pushDelete(row.blockId)
            rows.removeAt(rowIndex)
            pendingFocus.value = PendingFocus(above.blockId, joinOffset)
        }
    }

    fun onDeleteBlock(rowIndex: Int) {
        if (rowIndex <= 0) return
        val row = rows[rowIndex]
        if (row.isTitle) return
        val normalCount = rows.count { !it.isTitle }
        if (normalCount <= 1) return
        val focusIdx = (rowIndex - 1).coerceAtLeast(0)
        val focusRow = rows.getOrNull(focusIdx)
        // Move focus to the surviving block BEFORE removing this one so
        // the IME never sees a frame with no focused text field (which
        // is what causes the close-then-reopen flicker). Wrapped in
        // runCatching because requestFocus throws when the target block
        // isn't composed yet (offscreen); in that case the pendingFocus
        // path below still handles it, just with the old flicker.
        if (focusRow != null) {
            runCatching { focusRow.focusRequester.requestFocus() }
        }
        batcher.pushDelete(row.blockId)
        rows.removeAt(rowIndex)
        spawnGhost(row, rows[rowIndex - 1].blockId)
        if (focusRow != null) {
            pendingFocus.value = PendingFocus(focusRow.blockId, focusRow.textFieldValue.text.length)
        }
    }

    fun onMoveUp(rowIndex: Int) {
        if (rowIndex < 2) return
        val moved = rows[rowIndex]
        if (moved.isTitle) return
        val above1 = rows[rowIndex - 1]
        val above2 = rows[rowIndex - 2]
        batcher.pushMove(moved.blockId, above2.blockId, above1.blockId)
        rows.removeAt(rowIndex)
        rows.add(rowIndex - 1, moved)
    }

    fun onMoveDown(rowIndex: Int) {
        if (rowIndex >= rows.size - 1) return
        val moved = rows[rowIndex]
        if (moved.isTitle) return
        val below1 = rows[rowIndex + 1]
        val below2 = rows.getOrNull(rowIndex + 2)
        batcher.pushMove(moved.blockId, below1.blockId, below2?.blockId)
        rows.removeAt(rowIndex)
        rows.add(rowIndex + 1, moved)
    }

    fun onArrow(rowIndex: Int, goDown: Boolean): Boolean {
        if (goDown) {
            val next = rows.getOrNull(rowIndex + 1) ?: return false
            pendingFocus.value = PendingFocus(next.blockId, 0)
            return true
        } else {
            if (rowIndex <= 0) return false
            val prev = rows[rowIndex - 1]
            pendingFocus.value = PendingFocus(prev.blockId, prev.textFieldValue.text.length)
            return true
        }
    }

    fun queueEmptyBody() {
        val last = rows.lastOrNull() ?: return
        if (last.isTitle.not()) return
        val newId = UUID.randomUUID().toString().replace("-", "")
        batcher.pushAdd(newId, "", PositionToInsert.Between(last.blockId, null))
        rows.add(BlockRow(newId, false, ""))
    }

    /**
     * A remote letter was merged into this page's boss and the db mirror.
     * Bring the visible rows in line with the mirror, keeping the row
     * objects (so focus and cursor survive) for blocks that still exist.
     *
     * If local edits are still queued, flush first, then re-read the
     * mirror via [fetchMirror]. The mirror the caller captured may be
     * stale by the time the flush finishes; fetching after the flush
     * guarantees the rows we apply match the DB. On flush failure
     * [batcher] has already called onFailure (which reloads the page);
     * we do not apply rows on that path. Safe to call from any thread;
     * the row mutation runs on Main.
     */
    suspend fun applyRemoteRows(fetchMirror: suspend () -> List<BlockMirrorRow>) {
        if (batcher.hasPending()) {
            val result = batcher.flush()
            if (result.isFailure) return
        }
        val mirror = fetchMirror()
        withContext(Dispatchers.Main) { applyRemoteRowsNow(mirror) }
    }

    private fun applyRemoteRowsNow(mirrorRows: List<BlockMirrorRow>) {
        val byId = rows.associateBy { it.blockId }
        val next = mirrorRows.map { m ->
            val row = byId[m.blockId] ?: return@map BlockRow(m.blockId, m.isTitle, m.content)
            val current = row.textFieldValue
            if (current.text != m.content) {
                val end = m.content.length
                row.textFieldValue = current.copy(
                    text = m.content,
                    selection = TextRange(
                        current.selection.start.coerceAtMost(end),
                        current.selection.end.coerceAtMost(end)
                    )
                )
            }
            row
        }
        val same = next.size == rows.size && next.indices.all { next[it] === rows[it] }
        if (!same) {
            rows.clear()
            rows.addAll(next)
        }
    }

    fun clearPendingFocus() {
        pendingFocus.value = null
    }

    suspend fun flush(): Result<Unit> = batcher.flush()

    fun flushAsync() = batcher.flushAsync()

    /**
     * Asynchronously flush pending edits, destroy [boss], and destroy [batcher].
     * Safe to call from Compose `onDispose` without blocking the UI thread.
     */
    fun destroyAfterFlushAsync(): Job = batcher.flushAndDestroyAsync {
        runCatching { boss.destroy() }
    }

    /**
     * Flush, destroy the yrs boss, and stop accepting further edits.
     * Call once this session has been replaced; the caller must await
     * completion before reading from the DB or calling anything else on [boss].
     */
    suspend fun destroyAfterFlush() {
        destroyAfterFlushAsync().join()
    }
}

// text_diff's getDiff can emit Insert/Delete where a Replace is correct
// (see text_diff/src/diff_logic.rs: "aX" -> "hello" -> Insert("hello", 0)).
// We verify the raw diff by applying it; if it doesn't reproduce newText,
// we build the diff from a code-point prefix/suffix match instead.
private fun computeDiff(oldText: String, newText: String): DiffResult {
    if (oldText == newText) return DiffResult.NoDiff
    val raw = getDiff(oldText, newText)
    if (applyDiffOnCodepoints(oldText, raw) == newText) return raw
    if (newText.startsWith(oldText)) {
        return DiffResult.Insert(
            newText.substring(oldText.length),
            oldText.codePointCount(0, oldText.length).toUInt()
        )
    }
    if (oldText.startsWith(newText)) {
        return DiffResult.Delete(
            oldText.substring(newText.length),
            newText.codePointCount(0, newText.length).toUInt()
        )
    }
    val a = oldText.codePoints().toArray()
    val b = newText.codePoints().toArray()
    var p = 0
    while (p < a.size && p < b.size && a[p] == b[p]) p++
    var s = 0
    while (s < a.size - p && s < b.size - p && a[a.size - 1 - s] == b[b.size - 1 - s]) s++
    val oldMid = String(a, p, a.size - p - s)
    val newMid = String(b, p, b.size - p - s)
    return DiffResult.Replace(oldMid, newMid, p.toUInt())
}

private fun applyDiffOnCodepoints(oldText: String, diff: DiffResult): String {
    val cps = oldText.codePoints().toArray().toMutableList()
    when (diff) {
        is DiffResult.Insert -> {
            val p = diff.v2.toInt().coerceIn(0, cps.size)
            val ins = diff.v1.codePoints().toArray()
            for (i in ins.indices) cps.add(p + i, ins[i])
        }
        is DiffResult.Delete -> {
            val p = diff.v2.toInt().coerceIn(0, cps.size)
            val del = diff.v1.codePoints().toArray()
            if (p + del.size > cps.size) return oldText
            for (i in del.indices) if (cps[p + i] != del[i]) return oldText
            repeat(del.size) { cps.removeAt(p) }
        }
        is DiffResult.Replace -> {
            val p = diff.position.toInt().coerceIn(0, cps.size)
            val oldCp = diff.oldText.codePoints().toArray()
            if (p + oldCp.size > cps.size) return oldText
            for (i in oldCp.indices) if (cps[p + i] != oldCp[i]) return oldText
            val newCp = diff.newText.codePoints().toArray()
            repeat(oldCp.size) { cps.removeAt(p) }
            for (i in newCp.indices) cps.add(p + i, newCp[i])
        }
        DiffResult.NoDiff -> return oldText
    }
    return String(cps.toIntArray(), 0, cps.size)
}