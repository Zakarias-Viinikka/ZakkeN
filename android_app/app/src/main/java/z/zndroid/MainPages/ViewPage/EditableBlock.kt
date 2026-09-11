package z.zndroid.MainPages.ViewPage

import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.input.key.*
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import rustlib.my_yrs_lib.BossOfYrs
import z.zndroid.DocEvents.EditTextInBlock
import z.zndroid.DocEvents.EditTextInBlockCtx
import z.zndroid.DocEvents.RemoveBlock
import z.zndroid.DocEvents.RemoveBlockCtx
import rustlib.text_diff.getDiff

/**
 * A dedicated component for rendering and editing a single text block.
 * Handles styling (title vs body), live updates, and block deletion.
 */
@Composable
fun EditableBlock(
    state: BlockUiState,
    index: Int,
    boss: BossOfYrs,
    scope: CoroutineScope,
    onRefreshWithFocus: (String?, Int?) -> Unit
) {
    val isTitle = index == 0
    val textStyle = if (isTitle) {
        MaterialTheme.typography.headlineMedium
    } else {
        MaterialTheme.typography.bodyLarge
    }

    val debounceJob = remember { mutableStateOf<Job?>(null) }
    var showSlashPopup by remember { mutableStateOf(false) }
    var showLongPressMenu by remember { mutableStateOf(false) }

    Card(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 4.dp)
            .combinedClickable(
                onClick = { /* Normal click focus happens via TextField */ },
                onLongClick = { showLongPressMenu = true }
            ),
        elevation = CardDefaults.cardElevation(defaultElevation = 2.dp)
    ) {
        Box(modifier = Modifier.padding(16.dp)) {
            BasicTextField(
                value = state.textFieldValue,
                onValueChange = { newValue ->
                    val oldText = state.text
                    val newText = newValue.text
                    
                    // Slash command detection: subtle popup
                    if (newText.startsWith("/") && !oldText.startsWith("/")) {
                        showSlashPopup = true
                    } else if (!newText.startsWith("/")) {
                        showSlashPopup = false
                    }

                    if (isItTimeToMakeANewBlock(oldText, newText)) {
                        // 1. Logic: Triple linebreak triggers ManyEntersLeadsToManyBlocks sequentially.
                        scope.launch {
                            try {
                                ManyEntersLeadsToManyBlocks(
                                    boss = boss,
                                    state = state,
                                    newText = newText,
                                    nextPosition = index + 1,
                                    onUpdate = { newBlockId -> onRefreshWithFocus(newBlockId, null) }
                                )
                            } catch (e: Exception) {
                                // Handled by inner execute calls
                            }
                        }
                    } else {
                        // 2. Buffered update
                        val diff = getDiff(oldText, newText)
                        state.diffBuffer.add(diff)
                        state.textFieldValue = newValue
                        
                        debounceJob.value?.cancel()
                        debounceJob.value = scope.launch {
                            delay(500)
                            EditTextInBlock.flushBuffer(boss, state)
                        }
                    }
                },
                modifier = Modifier
                    .fillMaxWidth()
                    .focusRequester(state.focusRequester)
                    .onPreviewKeyEvent { keyEvent ->
                        if (keyEvent.type == KeyEventType.KeyDown) {
                            when (keyEvent.key) {
                                Key.Enter -> {
                                    if (!keyEvent.isShiftPressed) {
                                        val cursor = state.textFieldValue.selection.start
                                        scope.launch {
                                            splitBlock(boss, state, cursor, index + 1) { newBlockId ->
                                                onRefreshWithFocus(newBlockId, 0)
                                            }
                                        }
                                        return@onPreviewKeyEvent true
                                    }
                                }
                                Key.Backspace -> {
                                    val cursor = state.textFieldValue.selection.start
                                    // Merge if cursor is at the very beginning of the block
                                    if (cursor == 0) {
                                        scope.launch {
                                            mergeWithPreviousBlock(boss, index) { targetId, cursorAt ->
                                                onRefreshWithFocus(targetId, cursorAt)
                                            }
                                        }
                                        return@onPreviewKeyEvent true
                                    }
                                }
                            }
                        }
                        false
                    },
                textStyle = textStyle.copy(color = MaterialTheme.colorScheme.onSurface),
                cursorBrush = SolidColor(MaterialTheme.colorScheme.primary),
                decorationBox = { innerTextField ->
                    if (state.text.isEmpty()) {
                        Text(
                            text = "Type '/' for commands...",
                            style = textStyle,
                            color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.3f)
                        )
                    }
                    innerTextField()
                }
            )

            // Subtle Slash Popup
            if (showSlashPopup) {
                Card(
                    modifier = Modifier
                        .align(Alignment.BottomStart)
                        .padding(top = 40.dp),
                    elevation = CardDefaults.cardElevation(defaultElevation = 8.dp),
                    colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface)
                ) {
                    Text(
                        "Actions...",
                        modifier = Modifier.padding(8.dp).combinedClickable(
                            onClick = { 
                                showSlashPopup = false
                                showLongPressMenu = true 
                            }
                        ),
                        style = MaterialTheme.typography.bodySmall
                    )
                }
            }
        }
    }

    // Long Press Action Menu
    if (showLongPressMenu) {
        BlockActionMenu(
            blockText = state.text,
            onDismiss = { showLongPressMenu = false }
        )
    }
}

@Composable
fun BlockActionMenu(blockText: String, onDismiss: () -> Unit) {
    AlertDialog(
        onDismissRequest = onDismiss,
        confirmButton = {},
        title = {
            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceEvenly
                ) {
                    Button(onClick = { /* Future: Heading */ }) { Text("H1") }
                    Button(onClick = { /* Future: Todo */ }) { Text("Todo") }
                    Button(onClick = { /* Future: Delete */ }) { Text("Del") }
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
