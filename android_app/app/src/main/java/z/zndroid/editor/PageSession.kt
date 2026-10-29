package z.zndroid.editor

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.input.TextFieldValue
import rustlib.my_yrs_lib.BossOfYrs
import rustlib.my_yrs_lib.PositionToInsert
import rustlib.text_diff.getDiff
import z.zndroid.db.reads.BlockMirrorRow
import java.util.UUID

class BlockRow(
    val blockId: String,
    val isTitle: Boolean,
    initialText: String
) {
    var textFieldValue by mutableStateOf(TextFieldValue(initialText))
    val focusRequester = FocusRequester()
}

data class PendingFocus(val blockId: String, val offset: Int)

class PageSession(
    val boss: BossOfYrs,
    val pageId: String,
    val isMainMenu: Boolean,
    mirrorRows: List<BlockMirrorRow>,
    onFailure: () -> Unit
) {
    val rows = mutableStateListOf<BlockRow>()
    val pendingFocus = mutableStateOf<PendingFocus?>(null)
    private val batcher = PageBatcher(boss, pageId, isMainMenu, onFailure)

    init {
        for (row in mirrorRows) {
            rows.add(BlockRow(row.blockId, row.isTitle, row.content))
        }
    }

    fun onInput(rowIndex: Int, newValue: TextFieldValue) {
        val row = rows[rowIndex]
        val old = row.textFieldValue
        if (newValue.text == old.text) {
            row.textFieldValue = newValue
            return
        }
        val diff = getDiff(old.text, newValue.text)
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
            batcher.pushEdit(row.blockId, getDiff(text, head))
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

    fun onBackspaceAtStart(rowIndex: Int) {
        val row = rows[rowIndex]
        if (row.isTitle) return
        val aboveIndex = rowIndex - 1
        if (aboveIndex < 0) return
        val above = rows[aboveIndex]
        if (above.isTitle) return

        if (row.textFieldValue.text.isEmpty()) {
            batcher.pushDelete(row.blockId)
            rows.removeAt(rowIndex)
            pendingFocus.value = PendingFocus(above.blockId, above.textFieldValue.text.length)
        } else if (above.textFieldValue.text.isEmpty()) {
            batcher.pushDelete(above.blockId)
            rows.removeAt(aboveIndex)
            pendingFocus.value = PendingFocus(row.blockId, 0)
        } else {
            val combined = above.textFieldValue.text + row.textFieldValue.text
            val joinOffset = above.textFieldValue.text.length
            batcher.pushEdit(above.blockId, getDiff(above.textFieldValue.text, combined))
            batcher.pushDelete(row.blockId)
            above.textFieldValue = above.textFieldValue.copy(text = combined)
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
        batcher.pushDelete(row.blockId)
        rows.removeAt(rowIndex)
        val focusIdx = (rowIndex - 1).coerceAtLeast(0)
        if (focusIdx < rows.size) {
            val f = rows[focusIdx]
            pendingFocus.value = PendingFocus(f.blockId, f.textFieldValue.text.length)
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

    fun clearPendingFocus() {
        pendingFocus.value = null
    }

    suspend fun closeAndFlush(): Result<Unit> {
        return batcher.closeAndFlush()
    }

    fun closeAndFlushAsync() {
        batcher.closeAndFlushAsync()
    }
}
