package z.zndroid.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Snackbar
import androidx.compose.material3.SnackbarDuration
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.collectLatest

/**
 * Global Manager for showing popups (Snackbars) from anywhere in the app.
 */
object GlobalPopupManager {
    private val messages = MutableSharedFlow<String>()

    /**
     * Shows a popup with the given message.
     */
    suspend fun show(message: String) {
        messages.emit(message)
    }

    internal val events = messages
}

@Composable
fun GlobalPopup() {
    val state = remember { SnackbarHostState() }

    LaunchedEffect(Unit) {
        GlobalPopupManager.events.collectLatest { message ->
            state.showSnackbar(
                message = message,
                actionLabel = "Dismiss",
                duration = SnackbarDuration.Indefinite
            )
        }
    }

    Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.BottomCenter) {
        SnackbarHost(
            hostState = state,
            modifier = Modifier
                .statusBarsPadding()
                .navigationBarsPadding()
                .padding(16.dp)
        ) { data ->
            Snackbar(
                modifier = Modifier.clickable { data.dismiss() },
                action = {
                    TextButton(onClick = { data.performAction() }) {
                        Text(data.visuals.actionLabel ?: "Dismiss")
                    }
                },
                containerColor = MaterialTheme.colorScheme.errorContainer,
                contentColor = MaterialTheme.colorScheme.onErrorContainer,
                actionContentColor = MaterialTheme.colorScheme.error,
                shape = MaterialTheme.shapes.medium
            ) {
                Text(text = data.visuals.message)
            }
        }
    }
}
