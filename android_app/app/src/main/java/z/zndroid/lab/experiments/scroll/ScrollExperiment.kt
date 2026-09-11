package z.zndroid.lab.experiments.scroll

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.gestures.scrollBy
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.delay

/**
 * A Lab-only page demonstrating edge-scrolling during multi-select.
 */
@Composable
fun ScrollExperiment(onExit: () -> Unit) {
    val listState = rememberLazyListState()
    val trailPoints = remember { mutableStateListOf<Offset>() }
    
    var containerHeight by remember { mutableStateOf(0f) }
    var currentTouchPos by remember { mutableStateOf<Offset?>(null) }
    
    // Auto-scroll loop
    LaunchedEffect(currentTouchPos) {
        val pos = currentTouchPos ?: return@LaunchedEffect
        while (true) {
            val threshold = containerHeight * 0.15f
            if (pos.y < threshold) {
                listState.scrollBy(-20f)
            } else if (pos.y > containerHeight - threshold) {
                listState.scrollBy(20f)
            } else {
                break
            }
            delay(16)
        }
    }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .onGloballyPositioned { containerHeight = it.size.height.toFloat() }
    ) {
        LazyColumn(
            state = listState,
            modifier = Modifier.fillMaxSize().padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            item {
                Text("SCROLL EXPERIMENT", style = MaterialTheme.typography.headlineLarge)
                Text("Drag finger near top/bottom edges to auto-scroll.", 
                    style = MaterialTheme.typography.bodySmall)
                HorizontalDivider(modifier = Modifier.padding(vertical = 16.dp))
            }
            
            items(100) { i ->
                Card(
                    modifier = Modifier.fillMaxWidth(),
                    colors = CardDefaults.cardColors(
                        containerColor = MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.3f)
                    )
                ) {
                    Text(
                        "Experimental Block #$i\n" + 
                        "Placeholder text for scrolling test. ".repeat(2),
                        modifier = Modifier.padding(16.dp)
                    )
                }
            }
        }

        Canvas(
            modifier = Modifier
                .fillMaxSize()
                .pointerInput(Unit) {
                    detectDragGestures(
                        onDragStart = { trailPoints.clear() },
                        onDragEnd = { 
                            trailPoints.clear()
                            currentTouchPos = null
                        },
                        onDragCancel = { 
                            trailPoints.clear()
                            currentTouchPos = null
                        },
                        onDrag = { change, _ ->
                            currentTouchPos = change.position
                            trailPoints.add(change.position)
                            if (trailPoints.size > 25) trailPoints.removeAt(0)
                        }
                    )
                }
        ) {
            if (trailPoints.size > 1) {
                for (i in 0 until trailPoints.size - 1) {
                    val alpha = (i.toFloat() / trailPoints.size)
                    drawLine(
                        color = Color.Magenta.copy(alpha = alpha),
                        start = trailPoints[i],
                        end = trailPoints[i + 1],
                        strokeWidth = 12f
                    )
                }
            }
        }

        Button(
            onClick = onExit,
            modifier = Modifier.align(Alignment.TopEnd).padding(16.dp)
        ) {
            Text("Back to Core")
        }
    }
}
