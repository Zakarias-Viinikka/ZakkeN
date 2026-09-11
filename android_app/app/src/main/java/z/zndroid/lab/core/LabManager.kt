package z.zndroid.lab.core

import androidx.compose.animation.core.*
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import z.zndroid.lab.experiments.scroll.ScrollExperiment
import z.zndroid.lab.experiments.selection.SelectionTrailOverlay
import z.zndroid.lab.experiments.undoredo.UndoRedoTopBar

/**
 * The entry point for the Lab UI.
 * Orchestrates navigation between experiments and production overlays.
 */
@Composable
fun LabContainer(content: @Composable BoxScope.() -> Unit) {
    var showSmartMenu by remember { mutableStateOf(false) }
    
    Box(modifier = Modifier.fillMaxSize()) {
        when (val route = LabNavigator.currentRoute) {
            is LabNavigator.Route.Production -> {
                // 1. The Real App Content
                content()

                // 2. Experimental Overlays
                if (LabNavigator.isUndoRedoEnabled) {
                    Box(modifier = Modifier.align(Alignment.TopCenter)) {
                        UndoRedoTopBar()
                    }
                }

                if (LabNavigator.isSelectionModeEnabled) {
                    SelectionTrailOverlay()
                }

                // 3. Smart UI Controls
                SmartButton(
                    isShining = LabNavigator.isSelectionModeEnabled,
                    onClick = { showSmartMenu = !showSmartMenu }
                )

                if (showSmartMenu) {
                    SmartMenu(
                        onDismiss = { showSmartMenu = false },
                        onNavigateToScroll = { 
                            LabNavigator.navigateTo(LabNavigator.Route.ScrollLab)
                            showSmartMenu = false
                        }
                    )
                }
            }
            is LabNavigator.Route.ScrollLab -> {
                ScrollExperiment(onExit = { 
                    LabNavigator.navigateTo(LabNavigator.Route.Production)
                })
            }
        }
    }
}

@Composable
private fun SmartButton(isShining: Boolean, onClick: () -> Unit) {
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
                if (isShining) Color.Magenta.copy(alpha = shineAlpha) 
                else MaterialTheme.colorScheme.tertiary
            )
            .padding(4.dp)
    ) {
        IconButton(
            onClick = onClick,
            modifier = Modifier.fillMaxSize()
        ) {
            Text(
                text = if (isShining) "✨" else "🧪",
                style = MaterialTheme.typography.headlineSmall
            )
        }
    }
}

@Composable
private fun SmartMenu(onDismiss: () -> Unit, onNavigateToScroll: () -> Unit) {
    Card(
        modifier = Modifier
            .align(Alignment.BottomEnd)
            .padding(bottom = 90.dp, end = 16.dp)
            .width(220.dp),
        elevation = CardDefaults.cardElevation(defaultElevation = 12.dp)
    ) {
        Column(modifier = Modifier.padding(8.dp)) {
            Text("Lab Controls", style = MaterialTheme.typography.labelLarge)
            HorizontalDivider(modifier = Modifier.padding(vertical = 4.dp))
            
            Row(verticalAlignment = Alignment.CenterVertically) {
                Checkbox(
                    checked = LabNavigator.isUndoRedoEnabled, 
                    onCheckedChange = { LabNavigator.isUndoRedoEnabled = it }
                )
                Text("Undo Bar")
            }
            
            Row(verticalAlignment = Alignment.CenterVertically) {
                Checkbox(
                    checked = LabNavigator.isSelectionModeEnabled, 
                    onCheckedChange = { LabNavigator.isSelectionModeEnabled = it }
                )
                Text("Multi-Select")
            }

            Button(
                onClick = onNavigateToScroll,
                modifier = Modifier.fillMaxWidth().padding(top = 8.dp)
            ) {
                Text("Scroll Experiment")
            }
            
            Button(
                onClick = onDismiss,
                modifier = Modifier.fillMaxWidth().padding(top = 4.dp),
                colors = ButtonDefaults.buttonColors(containerColor = MaterialTheme.colorScheme.secondary)
            ) {
                Text("Close")
            }
        }
    }
}
