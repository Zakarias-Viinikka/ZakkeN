package z.zndroid.MainPages.ViewPage

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue

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
     * The current text being shown in the UI.
     * This is the "signal" that Compose observes for re-rendering.
     */
    var text by mutableStateOf(initialText)

    /**
     * Tracks the text that has been successfully persisted to the DB/CRDT.
     * Used for computing diffs on the next keystroke.
     */
    var lastCommittedText: String = initialText

    /**
     * The metadata for this block.
     */
    var metadata by mutableStateOf(initialMetadata)
}
