package z.zndroid.MainPages.ViewPage

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material3.LocalTextStyle
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import rustlib.my_yrs_lib.BossOfYrs
import z.zndroid.DocEvents.EditTextInBlock
import z.zndroid.DocEvents.EditTextInBlockCtx

/**
 * A dedicated component for rendering and editing a single text block.
 * Each block has its own state and lifecycle for calling EditTextInBlock.
 */
@Composable
fun EditableBlock(
    state: BlockUiState,
    boss: BossOfYrs,
    scope: CoroutineScope
) {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 4.dp)
    ) {
        BasicTextField(
            value = state.text,
            onValueChange = { newValue ->
                // 1. Update the local signal immediately
                val oldText = state.lastCommittedText
                state.text = newValue

                // 2. Fire the persistence event
                scope.launch {
                    EditTextInBlock.execute(
                        EditTextInBlockCtx(
                            boss = boss,
                            blockId = state.blockId,
                            oldText = oldText,
                            newText = newValue
                        )
                    ).onSuccess {
                        // 3. Update the commit tracker so the next diff is accurate
                        state.lastCommittedText = newValue
                    }
                }
            },
            modifier = Modifier.fillMaxWidth(),
            textStyle = LocalTextStyle.current.copy(
                color = MaterialTheme.colorScheme.onSurface,
                fontSize = MaterialTheme.typography.bodyLarge.fontSize
            ),
            cursorBrush = SolidColor(MaterialTheme.colorScheme.primary)
        )
    }
}
