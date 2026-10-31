package com.example.feellabs.notes

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.input.TextFieldLineLimits
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.text.TextStyle

@Composable
fun BlockEditor(
    state: TextFieldState,
    textStyle: TextStyle,
    cursorColor: Color,
    canMergeUp: Boolean,
    onSplit: (before: String, after: String) -> Unit,
    onMergeUp: () -> Unit,
    modifier: Modifier = Modifier,
) {
    LaunchedEffect(state) {
        snapshotFlow { state.text.toString() }.collect { text ->
            val nl = text.indexOf('\n')
            if (nl >= 0) {
                val before = text.substring(0, nl)
                val after = text.substring(nl + 1)
                state.edit { replace(0, length, before) }
                onSplit(before, after)
            }
        }
    }

    BasicTextField(
        state = state,
        modifier = modifier
            .fillMaxWidth()
            .onPreviewKeyEvent { ev ->
                if (ev.type != KeyEventType.KeyDown) return@onPreviewKeyEvent false
                if (ev.key == Key.Backspace && canMergeUp) {
                    val sel = state.selection
                    if (sel.start == 0 && sel.end == 0) {
                        onMergeUp()
                        return@onPreviewKeyEvent true
                    }
                }
                false
            },
        textStyle = textStyle,
        cursorBrush = SolidColor(cursorColor),
        lineLimits = TextFieldLineLimits.MultiLine(
            minHeightInLines = 1,
            maxHeightInLines = Int.MAX_VALUE,
        ),
    )
}
