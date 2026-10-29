package z.zndroid.MainPages.ViewPage

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.withFrameNanos
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.isShiftPressed
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.delay
import z.zndroid.editor.BlockRow
import z.zndroid.editor.PageSession

@Composable
fun EditableBlock(
    row: BlockRow,
    index: Int,
    session: PageSession,
    listState: LazyListState
) {
    val isTitle = row.isTitle
    val textStyle = if (isTitle) MaterialTheme.typography.headlineMedium else MaterialTheme.typography.bodyLarge
    var showMenu by remember { mutableStateOf(false) }

    LaunchedEffect(session.pendingFocus.value) {
        val req = session.pendingFocus.value ?: return@LaunchedEffect
        if (req.blockId != row.blockId) return@LaunchedEffect
        runCatching { listState.animateScrollToItem(index) }
        for (attempt in 0..5) {
            try {
                withFrameNanos { }
                row.focusRequester.requestFocus()
                row.textFieldValue = row.textFieldValue.copy(selection = TextRange(req.offset))
                session.clearPendingFocus()
                return@LaunchedEffect
            } catch (_: IllegalStateException) {
                delay(16)
            }
        }
    }

    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 4.dp),
        verticalAlignment = Alignment.Top
    ) {
        Box(
            modifier = Modifier
                .width(24.dp)
                .padding(top = 4.dp)
                .clickable { showMenu = true },
            contentAlignment = Alignment.Center
        ) {
            Text(
                text = "::",
                style = MaterialTheme.typography.bodyLarge,
                color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.35f)
            )
        }

        BasicTextField(
            value = row.textFieldValue,
            onValueChange = { newValue ->
                if (newValue.text.contains('\n')) {
                    val cleanText = newValue.text.replace("\n", "")
                    val cursor = newValue.selection.start.coerceAtMost(cleanText.length)
                    session.onEnter(index, cursor)
                    return@BasicTextField
                }
                if (newValue.text == row.textFieldValue.text) {
                    row.textFieldValue = newValue
                } else {
                    session.onInput(index, newValue)
                }
            },
            modifier = Modifier
                .weight(1f)
                .focusRequester(row.focusRequester)
                .onPreviewKeyEvent { event ->
                    if (event.type != KeyEventType.KeyDown) return@onPreviewKeyEvent false
                    when (event.key) {
                        Key.Enter -> {
                            if (!event.isShiftPressed) {
                                val cursor = row.textFieldValue.selection.start
                                session.onEnter(index, cursor)
                                true
                            } else false
                        }
                        Key.Backspace -> {
                            val cursor = row.textFieldValue.selection.start
                            if (cursor == 0 && index > 0) {
                                session.onBackspaceAtStart(index)
                                true
                            } else false
                        }
                        Key.DirectionUp -> {
                            val cursor = row.textFieldValue.selection.start
                            if (cursor == 0 && !event.isShiftPressed) {
                                session.onArrow(index, goDown = false)
                            } else false
                        }
                        Key.DirectionDown -> {
                            val cursor = row.textFieldValue.selection.start
                            val len = row.textFieldValue.text.length
                            if (cursor == len && !event.isShiftPressed) {
                                session.onArrow(index, goDown = true)
                            } else false
                        }
                        else -> false
                    }
                },
            textStyle = textStyle.copy(color = MaterialTheme.colorScheme.onSurface),
            cursorBrush = SolidColor(MaterialTheme.colorScheme.primary),
            decorationBox = { innerTextField ->
                if (row.textFieldValue.text.isEmpty()) {
                    Text(
                        text = if (isTitle) "Page Title" else "Type '/' for commands...",
                        style = textStyle,
                        color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.3f)
                    )
                }
                innerTextField()
            }
        )
    }

    if (showMenu) {
        BlockActionMenu(
            blockText = row.textFieldValue.text,
            canMoveUp = index >= 2,
            canMoveDown = index < session.rows.size - 1,
            canDelete = !isTitle && session.rows.count { !it.isTitle } > 1,
            onDelete = {
                session.onDeleteBlock(index)
                showMenu = false
            },
            onMoveUp = {
                session.onMoveUp(index)
                showMenu = false
            },
            onMoveDown = {
                session.onMoveDown(index)
                showMenu = false
            },
            onDismiss = { showMenu = false }
        )
    }
}

@Composable
fun BlockActionMenu(
    blockText: String,
    canMoveUp: Boolean,
    canMoveDown: Boolean,
    canDelete: Boolean,
    onDelete: () -> Unit,
    onMoveUp: () -> Unit,
    onMoveDown: () -> Unit,
    onDismiss: () -> Unit
) {
    AlertDialog(
        onDismissRequest = onDismiss,
        confirmButton = {},
        title = {
            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceEvenly
                ) {
                    Button(onClick = { }) { Text("H1") }
                    Button(onClick = { }) { Text("Todo") }
                    Button(onClick = onDelete, enabled = canDelete) { Text("Del") }
                }
                Spacer(modifier = Modifier.height(8.dp))
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceEvenly
                ) {
                    Button(onClick = onMoveUp, enabled = canMoveUp) { Text("Up") }
                    Button(onClick = onMoveDown, enabled = canMoveDown) { Text("Down") }
                }
                Spacer(modifier = Modifier.height(16.dp))
                Card(
                    modifier = Modifier.fillMaxWidth(),
                    colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceVariant)
                ) {
                    Text(
                        text = blockText,
                        modifier = Modifier.padding(16.dp),
                        style = MaterialTheme.typography.bodyLarge
                    )
                }
            }
        }
    )
}