package z.special_menu

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.OffsetMapping
import androidx.compose.ui.text.input.TransformedText
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.unit.sp

const val BADGE_LEN = 3

class MarkerVisualTransformation(
    private val markers: List<TextMarker>,
    private val highlightRange: Pair<Int, Int>?
) : VisualTransformation {
    override fun filter(text: AnnotatedString): TransformedText {
        val rawText = text.text
        val sortedMarkers = markers
            .filter { it.characterIndex in 0..rawText.length }
            .sortedBy { it.characterIndex }

        val validHighlight = highlightRange?.let { (from, to) ->
            val start = from.coerceIn(0, rawText.length)
            val end = to.coerceIn(start, rawText.length)
            if (start < end) Pair(start, end) else null
        }

        if (sortedMarkers.isEmpty() && validHighlight == null) {
            return TransformedText(text, OffsetMapping.Identity)
        }

        val boundaries = sortedSetOf<Int>()
        boundaries.add(0)
        boundaries.add(rawText.length)
        for (m in sortedMarkers) {
            boundaries.add(m.characterIndex.coerceIn(0, rawText.length))
        }
        validHighlight?.let { (from, to) ->
            boundaries.add(from)
            boundaries.add(to)
        }

        val sortedBoundaries = boundaries.toList()
        val builder = AnnotatedString.Builder()

        for (i in 0 until sortedBoundaries.size - 1) {
            val start = sortedBoundaries[i]
            val end = sortedBoundaries[i + 1]

            // Emit badges located at `start`
            for (m in sortedMarkers.filter { it.characterIndex == start }) {
                val badgeBg = if (m.option == "1") Color(0xFF1976D2) else Color(0xFF388E3C)
                builder.pushStyle(
                    SpanStyle(
                        color = Color.White,
                        background = badgeBg,
                        fontWeight = FontWeight.Bold,
                        fontSize = 13.sp
                    )
                )
                builder.append(" ${m.option} ")
                builder.pop()
            }

            // Emit slice of rawText
            if (start < end) {
                val isHighlighted = validHighlight != null && start >= validHighlight.first && end <= validHighlight.second
                if (isHighlighted) {
                    builder.pushStyle(SpanStyle(background = Color(0xFFBBDEFB)))
                    builder.append(rawText.substring(start, end))
                    builder.pop()
                } else {
                    builder.append(rawText.substring(start, end))
                }
            }
        }

        // Emit badges at the very end of rawText
        val finalIdx = rawText.length
        for (m in sortedMarkers.filter { it.characterIndex == finalIdx }) {
            val badgeBg = if (m.option == "1") Color(0xFF1976D2) else Color(0xFF388E3C)
            builder.pushStyle(
                SpanStyle(
                    color = Color.White,
                    background = badgeBg,
                    fontWeight = FontWeight.Bold,
                    fontSize = 13.sp
                )
            )
            builder.append(" ${m.option} ")
            builder.pop()
        }

        val transformedAnnotated = builder.toAnnotatedString()

        val offsetMapping = object : OffsetMapping {
            override fun originalToTransformed(offset: Int): Int {
                val clamped = offset.coerceIn(0, rawText.length)
                var shift = 0
                for (m in sortedMarkers) {
                    if (clamped > m.characterIndex) {
                        shift += BADGE_LEN
                    }
                }
                return (clamped + shift).coerceIn(0, transformedAnnotated.length)
            }

            override fun transformedToOriginal(offset: Int): Int {
                val clamped = offset.coerceIn(0, transformedAnnotated.length)
                var currentTransformed = 0
                var currentOriginal = 0

                for (m in sortedMarkers) {
                    val delta = m.characterIndex - currentOriginal
                    if (clamped <= currentTransformed + delta) {
                        return (currentOriginal + (clamped - currentTransformed)).coerceIn(0, rawText.length)
                    }
                    currentTransformed += delta
                    currentOriginal += delta

                    if (clamped <= currentTransformed + BADGE_LEN) {
                        return currentOriginal.coerceIn(0, rawText.length)
                    }
                    currentTransformed += BADGE_LEN
                }

                return (currentOriginal + (clamped - currentTransformed)).coerceIn(0, rawText.length)
            }
        }

        return TransformedText(transformedAnnotated, offsetMapping)
    }
}
