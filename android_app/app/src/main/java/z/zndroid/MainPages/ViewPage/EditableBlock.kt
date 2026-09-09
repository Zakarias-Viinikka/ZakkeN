package z.zndroid.MainPages.ViewPage

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.input.key.*
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import rustlib.my_yrs_lib.BossOfYrs
import z.zndroid.DocEvents.EditTextInBlock
import z.zndroid.DocEvents.EditTextInBlockCtx
import z.zndroid.DocEvents.RemoveBlock
import z.zndroid.DocEvents.RemoveBlockCtx

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
    onRefresh: () -> Unit
) {
    val isTitle = index == 0
    val textStyle = if (isTitle) {
        MaterialTheme.typography.headlineMedium
    } else {
        MaterialTheme.typography.bodyLarge
    }

    Card(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 4.dp),
        elevation = CardDefaults.cardElevation(defaultElevation = 2.dp)
    ) {
        Box(modifier = Modifier.padding(16.dp)) {
            BasicTextField(
                value = state.text,
                onValueChange = { newValue ->
                    val oldText = state.lastCommittedText
                    
                    if (isItTimeToMakeANewBlock(oldText, newValue)) {
                        // 1. Logic: Triple linebreak triggers ManyEntersLeadsToManyBlocks sequentially.
                        scope.launch {
                            try {
                                ManyEntersLeadsToManyBlocks(
                                    boss = boss,
                                    blockId = state.blockId,
                                    oldText = oldText,
                                    newText = newValue,
                                    nextPosition = index + 1,
                                    onUpdate = onRefresh
                                )
                                // Success: Update local state if needed (though updateUI will reload anyway)
                            } catch (e: Exception) {
                                // Handled by inner execute calls, but good to have here
                            }
                        }
                    } else {
                        // 2. Normal character-by-character update
                        state.text = newValue
                        scope.launch {
                            EditTextInBlock.execute(
                                EditTextInBlockCtx(
                                    boss = boss,
                                    blockId = state.blockId,
                                    oldText = oldText,
                                    newText = newValue
                                )
                            ).onSuccess {
                                state.lastCommittedText = newValue
                            }
                        }
                    }
                },
                modifier = Modifier
                    .fillMaxWidth()
                    .onPreviewKeyEvent { keyEvent ->
                        // 3. Logic: Pressing backspace in an empty block deletes it
                        if (keyEvent.key == Key.Backspace && state.text.isEmpty() && keyEvent.type == KeyEventType.KeyDown) {
                            scope.launch {
                                RemoveBlock.execute(RemoveBlockCtx(
                                    boss = boss,
                                    blockId = state.blockId,
                                    position = index.toUInt()
                                )).onSuccess {
                                    onRefresh()
                                }
                            }
                            true
                        } else {
                            false
                        }
                    },
                textStyle = textStyle.copy(color = MaterialTheme.colorScheme.onSurface),
                cursorBrush = SolidColor(MaterialTheme.colorScheme.primary)
            )
        }
    }
}
