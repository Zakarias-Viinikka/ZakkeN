package z.zndroid.lab.experiments.undoredo

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

/**
 * Undo/Redo Lab Box Experiment.
 * A top bar with demo navigation arrows.
 */
@Composable
fun UndoRedoTopBar() {
    Surface(
        modifier = Modifier
            .fillMaxWidth()
            .height(60.dp),
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
