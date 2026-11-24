package z.special_menu

import androidx.compose.ui.text.input.TextFieldValue

data class TextMarker(
    val option: String,
    val blockId: Long,
    val characterIndex: Int
)

enum class SelectionAction {
    COPY,
    CUT,
    DELETE,
    CLEAR
}

data class SelectionSegment(
    val blockId: Long,
    val from: Int,
    val to: Int
)

data class DerivedSelection(
    val segments: List<SelectionSegment>,
    val copyText: String,
    val hasSelection: Boolean,
    val isSingleBlock: Boolean
) {
    fun highlightRangeFor(blockId: Long): Pair<Int, Int>? {
        if (!hasSelection) return null
        val seg = segments.firstOrNull { it.blockId == blockId } ?: return null
        if (seg.from >= seg.to) return null
        return Pair(seg.from, seg.to)
    }
}

fun shiftMarkerOnEdit(oldIdx: Int, oldText: String, newText: String): Int {
    if (oldText == newText) return oldIdx.coerceIn(0, newText.length)
    val oldLen = oldText.length
    val newLen = newText.length

    var p = 0
    while (p < oldLen && p < newLen && oldText[p] == newText[p]) {
        p++
    }

    var s = 0
    while (s < (oldLen - p) && s < (newLen - p) && oldText[oldLen - 1 - s] == newText[newLen - 1 - s]) {
        s++
    }

    val res = when {
        oldIdx <= p -> oldIdx
        oldIdx >= oldLen - s -> oldIdx + (newLen - oldLen)
        else -> p
    }
    return res.coerceIn(0, newLen)
}

fun deriveSelection(
    order: List<Long>,
    texts: Map<Long, TextFieldValue>,
    m1: TextMarker?,
    m2: TextMarker?
): DerivedSelection {
    if (m1 == null || m2 == null) {
        return DerivedSelection(emptyList(), "", hasSelection = false, isSingleBlock = false)
    }
    val idx1 = order.indexOf(m1.blockId)
    val idx2 = order.indexOf(m2.blockId)
    if (idx1 < 0 || idx2 < 0) {
        return DerivedSelection(emptyList(), "", hasSelection = false, isSingleBlock = false)
    }

    val (start, end) = if (idx1 < idx2 || (idx1 == idx2 && m1.characterIndex <= m2.characterIndex)) {
        m1 to m2
    } else {
        m2 to m1
    }

    val startOrderIdx = order.indexOf(start.blockId)
    val endOrderIdx = order.indexOf(end.blockId)

    if (start.blockId == end.blockId && start.characterIndex == end.characterIndex) {
        return DerivedSelection(emptyList(), "", hasSelection = false, isSingleBlock = true)
    }

    val segments = mutableListOf<SelectionSegment>()
    if (startOrderIdx == endOrderIdx) {
        val blockId = start.blockId
        val textLen = texts[blockId]?.text?.length ?: 0
        val from = start.characterIndex.coerceIn(0, textLen)
        val to = end.characterIndex.coerceIn(from, textLen)
        segments.add(SelectionSegment(blockId, from, to))
    } else {
        for (i in startOrderIdx..endOrderIdx) {
            val blockId = order[i]
            val textLen = texts[blockId]?.text?.length ?: 0
            when (i) {
                startOrderIdx -> {
                    val from = start.characterIndex.coerceIn(0, textLen)
                    segments.add(SelectionSegment(blockId, from, textLen))
                }
                endOrderIdx -> {
                    val to = end.characterIndex.coerceIn(0, textLen)
                    segments.add(SelectionSegment(blockId, 0, to))
                }
                else -> {
                    segments.add(SelectionSegment(blockId, 0, textLen))
                }
            }
        }
    }

    val copyPieces = mutableListOf<String>()
    for ((index, seg) in segments.withIndex()) {
        val fullText = texts[seg.blockId]?.text ?: ""
        val piece = if (seg.from < seg.to && seg.from <= fullText.length) {
            fullText.substring(seg.from, seg.to.coerceAtMost(fullText.length))
        } else {
            ""
        }
        val isEdge = index == 0 || index == segments.lastIndex
        if (isEdge && piece.isEmpty()) {
            continue
        }
        copyPieces.add(piece)
    }
    val copyText = copyPieces.joinToString("\n")
    val hasSelection = segments.any { it.from < it.to } || segments.size > 1

    return DerivedSelection(
        segments = segments,
        copyText = copyText,
        hasSelection = hasSelection,
        isSingleBlock = startOrderIdx == endOrderIdx
    )
}
