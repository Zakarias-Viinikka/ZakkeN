package z.zndroid.lab

import androidx.compose.animation.core.*
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.delay

/**
 * The Lab Container: A wrapper that adds experimental UI features.
 * Adheres to AI_LAB_RULE.md: Never affects production logic.
 */
@Composable
fun LabContainer(content: @Composable BoxScope.() -> Unit) {
    var showSmartMenu by remember { mutableStateOf(false) }
    var undoRedoVisible by remember { mutableStateOf(false) }
    var multiSelectMode by remember { mutableStateOf(false) }
    
    // Trail for Multi-Select
    val trailPoints = remember { mutableStateListOf<Offset>() }
    
    Box(modifier = Modifier.fillMaxSize()) {
        // 1. The Real App Content
        content()

        // 2. Undo/Redo Lab Box (Top)
        if (undoRedoVisible) {
            Surface(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(60.dp)
                    .align(Alignment.TopCenter),
                color = MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.9f),
                shadowElevation = 4.dp
            ) {
                Row(
                    modifier = Modifier.fillMaxSize(),
                    horizontalArrangement = Arrangement.SpaceEvenly,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    IconButton(onClick = { /* Demo only */ }) {
                        Text("←", style = MaterialTheme.typography.headlineLarge)
                    }
                    Text("Undo / Redo Lab", style = MaterialTheme.typography.titleMedium)
                    IconButton(onClick = { /* Demo only */ }) {
                        Text("→", style = MaterialTheme.typography.headlineLarge)
                    }
                }
            }
        }

        // 3. Multi-Select Trail Canvas
        if (multiSelectMode) {
            Canvas(
                modifier = Modifier
                    .fillMaxSize()
                    .pointerInput(Unit) {
                        detectDragGestures(
                            onDragStart = { trailPoints.clear() },
                            onDragEnd = { trailPoints.clear() },
                            onDragCancel = { trailPoints.clear() },
                            onDrag = { change, dragAmount ->
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

        // 4. Smart Button (Bottom Right)
        val infiniteTransition = rememberInfiniteTransition(label = "shining")
        val shineAlpha by infiniteTransition.animateFloat(
            initialValue = 0.4f,
            targetValue = 1.0f,
            animationSpec = infiniteRepeatable(
                animation = tween(1000, easing = LinearEasing),
                repeatMode = RepeatMode.Reverse
            ),
            label = "shineAlpha"
        )

        Box(
            modifier = Modifier
                .padding(16.dp)
                .size(64.dp)
                .align(Alignment.BottomEnd)
                .shadow(8.dp, CircleShape)
                .clip(CircleShape)
                .background(
                    if (multiSelectMode) Color.Magenta.copy(alpha = shineAlpha) 
                    else MaterialTheme.colorScheme.tertiary
                )
                .pointerInput(Unit) {
                    detectDragGestures(
                        onDrag = { _, _ -> /* Drag logic if needed */ }
                    )
                }
                .padding(4.dp)
        ) {
            IconButton(
                onClick = { showSmartMenu = !showSmartMenu },
                modifier = Modifier.fillMaxSize()
            ) {
                Text(
                    text = if (multiSelectMode) "✨" else "🧪",
                    style = MaterialTheme.typography.headlineSmall
                )
            }
        }

        // 5. Smart Menu Popup
        if (showSmartMenu) {
            Card(
                modifier = Modifier
                    .align(Alignment.BottomEnd)
                    .padding(bottom = 90.dp, end = 16.dp)
                    .width(200.dp),
                elevation = CardDefaults.cardElevation(defaultElevation = 12.dp)
            ) {
                Column(modifier = Modifier.padding(8.dp)) {
                    Text("Experimental Controls", style = MaterialTheme.typography.labelLarge)
                    Divider(modifier = Modifier.padding(vertical = 4.dp))
                    
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Checkbox(checked = undoRedoVisible, onCheckedChange = { undoRedoVisible = it })
                        Text("Undo/Redo Box")
                    }
                    
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Checkbox(checked = multiSelectMode, onCheckedChange = { multiSelectMode = it })
                        Text("Multi-Select Mode")
                    }
                    
                    Button(
                        onClick = { showSmartMenu = false },
                        modifier = Modifier.fillMaxWidth().padding(top = 8.dp)
                    ) {
                        Text("Close")
                    }
                }
            }
        }
    }
}
