package z.zndroid.markdown_conversion

//! Pure edit logic. No JS, no DOM. Offsets are UTF-16 code units in the OLD
//! document (what CodeMirror reports). Direct port of
//! cqrs_state_synchronization/web_interface/src/markdown_conversion/edit.rs

data class Change(
    val from: Int,
    val to: Int,
    val insert: String,
)

/** Apply CodeMirror's change list (all positions in OLD-document coordinates) to [old]. */
fun applyChanges(old: String, changes: List<Change>): String {
    if (changes.isEmpty()) return old
    val sorted = changes.sortedBy { it.from }
    val sb = StringBuilder(old.length + sorted.sumOf { it.insert.length })
    var pos = 0
    for (c in sorted) {
        val from = c.from.coerceIn(pos, old.length)
        val to = c.to.coerceIn(from, old.length)
        sb.append(old, pos, from)
        sb.append(c.insert)
        pos = to
    }
    sb.append(old, pos, old.length)
    return sb.toString()
}
