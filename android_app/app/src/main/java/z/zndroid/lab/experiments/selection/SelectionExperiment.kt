package z.zndroid.lab.experiments.selection

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.pointerInput

/**
 * Multi-Select Trail Experiment.
 * Draws a fading cyan trail following the user's finger.
 */
@Composable
fun SelectionTrailOverlay() {
    val trailPoints = remember { mutableStateListOf<Offset>() }

    Canvas(
        modifier = Modifier
            .fillMaxSize()
            .pointerInput(Unit) {
                detectDragGestures(
                    onDragStart = { trailPoints.clear() },
                    onDragEnd = { trailPoints.clear() },
                    onDragCancel = { trailPoints.clear() },
                    onDrag = { change, _ ->
                        trailPoints.add(change.position)
                        if (trailPoints.size > 20) trailPoints.removeAt(0)
                    }
                )
            }
    ) {
        if (trailPoints.size > 1) {
            for (i in 0 until trailPoints.size - 1) {
                val alpha = (i.toFloat() / trailPoints.size)
                drawLine(
                    color = Color.Cyan.copy(alpha = alpha),
                    start = trailPoints[i],
                    end = trailPoints[i + 1],
                    strokeWidth = 10f
                )
            }
        }
    }
}
