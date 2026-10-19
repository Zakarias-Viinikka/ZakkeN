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
import android.util.Log
import z.zndroid.BuildConfig
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.collectLatest

/**
 * Global Manager for showing popups (Snackbars) from anywhere in the app.
 *
 * Two entry points:
 *   - show(msg)      : informational popup; tester records but does not fail.
 *   - showError(msg) : error popup; tester fails the step that produced it.
 *
 * Error popups are rendered with an "ERROR: " prefix so the running-app a11y
 * tree distinguishes them without needing access to logcat.
 */
internal data class Popup(val text: String, val isError: Boolean)

object GlobalPopupManager {
    private val messages = MutableSharedFlow<Popup>()

    suspend fun show(message: String) {
        if (BuildConfig.DEBUG) {
            Log.i("GlobalPopup", message)
        }
        messages.emit(Popup(message, isError = false))
    }

    suspend fun showError(message: String) {
        if (BuildConfig.DEBUG) {
            Log.e("GlobalPopup", message)
        }
        val cleaned = message
            .removePrefix("ERROR: ")
            .removePrefix("Error: ")
            .removePrefix("error: ")
        messages.emit(Popup(cleaned, isError = true))
    }

    internal val events = messages
}

@Composable
fun GlobalPopup() {
    val state = remember { SnackbarHostState() }

    LaunchedEffect(Unit) {
        GlobalPopupManager.events.collectLatest { msg ->
            val display = if (msg.isError) "ERROR: ${msg.text}" else msg.text
            state.showSnackbar(
                message = display,
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