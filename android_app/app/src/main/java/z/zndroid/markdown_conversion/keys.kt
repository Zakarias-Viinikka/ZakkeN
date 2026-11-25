package z.zndroid.markdown_conversion

//! Key decisions. Pure Kotlin, no JS, no DOM. All offsets are UTF-16 code units
//! (what CodeMirror reports). Direct port of
//! cqrs_state_synchronization/web_interface/src/markdown_conversion/keys.rs

sealed class Intent {
    object NavUp : Intent()
    object NavDown : Intent()
    data class Split(val before: String, val after: String) : Intent()
    /** Backspace at position 0, or Delete in an empty block. */
    object MergeUp : Intent()
    /**
     * Enter on a title, or in an empty / whitespace-only block: not consumed,
     * CM inserts the newline, and the next input event carries was_enter.
     */
    object EnterWithoutSplit : Intent()
}

data class Decision(
    val intent: Intent,
    /** true => Kotlin handled it, CM must not. */
    val consume: Boolean,
)

private fun consume(intent: Intent): Decision = Decision(intent, true)
private fun enterWithoutSplit(): Decision = Decision(Intent.EnterWithoutSplit, false)

/** Split [text] around the UTF-16 range [from]..[to] (the selection is dropped). */
fun splitUtf16(text: String, from: Int, to: Int): Pair<String, String> {
    val f = from.coerceIn(0, text.length)
    val t = to.coerceIn(f, text.length)
    return text.substring(0, f) to text.substring(t)
}

/**
 * [text] is only called when the decision needs the document (Enter on a non-title).
 */
fun decide(
    key: String,
    anchor: Int,
    head: Int,
    docLen: Int,
    ranges: Int,
    isTitle: Boolean,
    text: () -> String,
): Decision? {
    if (ranges != 1) return null // multi-cursor: leave it to CM
    val empty = anchor == head
    return when (key) {
        "ArrowUp" -> if (empty && head == 0) consume(Intent.NavUp) else null
        "ArrowDown" -> if (empty && head == docLen) consume(Intent.NavDown) else null
        "Enter" -> {
            if (isTitle || docLen == 0) {
                enterWithoutSplit()
            } else {
                val full = text()
                if (full.isBlank()) {
                    enterWithoutSplit()
                } else {
                    val (before, after) = splitUtf16(full, minOf(anchor, head), maxOf(anchor, head))
                    consume(Intent.Split(before, after))
                }
            }
        }
        "Backspace" -> if (empty && head == 0) consume(Intent.MergeUp) else null
        "Delete" -> if (docLen == 0) consume(Intent.MergeUp) else null
        else -> null
    }
}
