package z.zndroid.MainPages.ViewPage

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.text.input.TextFieldValue

/**
 * Represents the UI-specific state for a single text block.
 * Similar to a Leptos struct with individual signals for text and metadata.
 */
class BlockUiState(
    val blockId: String,
    initialText: String,
    val initialMetadata: String = ""
) {
    /**
     * The current text and selection being shown in the UI.
     * We use TextFieldValue to track the cursor position for splitting/merging.
     */
    var textFieldValue by mutableStateOf(TextFieldValue(initialText))

    /**
     * Used to programmatically request focus for this block.
     */
    val focusRequester = FocusRequester()

    /**
     * Helper to get the current text string.
     */
    val text: String get() = textFieldValue.text

    /**
     * Tracks the text that has been successfully persisted to the DB/CRDT.
     * Used for computing diffs on the next keystroke when starting a new batch.
     */
    var lastPersistedText: String = initialText

    /**
     * Buffer of individual keystroke diffs waiting to be squashed and flushed.
     */
    val diffBuffer = mutableListOf<rustlib.text_diff.DiffResult>()

    /**
     * The metadata for this block.
     */
    var metadata by mutableStateOf(initialMetadata)
}
