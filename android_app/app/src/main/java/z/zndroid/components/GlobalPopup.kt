package z.zndroid.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.consumeWindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBars
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
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import android.util.Log
import z.zndroid.BuildConfig
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.receiveAsFlow

/**
 * Global Manager for showing popups (Snackbars) from anywhere in the app.
 *
 * Two entry points:
 *   - show(msg)      : informational popup.
 *   - showError(msg) : error popup; prefixed with "ERROR:" in the snackbar.
 *
 * Both are fire-and-forget: the backing channel is unbounded, so a popup
 * requested before [GlobalPopup] first composes (or while the activity is
 * being recreated) sits in the channel and is shown by the next collector.
 */
internal data class Popup(val text: String, val isError: Boolean)

object GlobalPopupManager {
    private val channel = Channel<Popup>(capacity = Channel.UNLIMITED)

    fun show(message: String) {
        if (BuildConfig.DEBUG) {
            Log.i("GlobalPopup", message)
        }
        channel.trySend(Popup(message, isError = false))
    }

    fun showError(message: String) {
        if (BuildConfig.DEBUG) {
            Log.e("GlobalPopup", message)
        }
        val cleaned = message
            .removePrefix("ERROR: ")
            .removePrefix("Error: ")
            .removePrefix("error: ")
        channel.trySend(Popup(cleaned, isError = true))
    }

    internal val events = channel.receiveAsFlow()
}

@Composable
fun GlobalPopup() {
    val state = remember { SnackbarHostState() }
    var currentIsError by remember { mutableStateOf(false) }

    LaunchedEffect(Unit) {
        GlobalPopupManager.events.collect { msg ->
            currentIsError = msg.isError
            val display = if (msg.isError) "ERROR: ${msg.text}" else msg.text
            state.showSnackbar(
                message = display,
                actionLabel = "Dismiss",
                duration = SnackbarDuration.Indefinite
            )
        }
    }

    val containerColor = if (currentIsError) {
        MaterialTheme.colorScheme.errorContainer
    } else {
        MaterialTheme.colorScheme.secondaryContainer
    }
    val contentColor = if (currentIsError) {
        MaterialTheme.colorScheme.onErrorContainer
    } else {
        MaterialTheme.colorScheme.onSecondaryContainer
    }
    val actionColor = if (currentIsError) {
        MaterialTheme.colorScheme.error
    } else {
        MaterialTheme.colorScheme.onSecondaryContainer
    }

    Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.BottomCenter) {
        SnackbarHost(
            hostState = state,
            modifier = Modifier
                .statusBarsPadding()
                .navigationBarsPadding()
                .consumeWindowInsets(WindowInsets.navigationBars)
                .imePadding()
                .padding(16.dp)
        ) { data ->
            Snackbar(
                modifier = Modifier.clickable { data.dismiss() },
                action = {
                    TextButton(onClick = { data.performAction() }) {
                        Text(data.visuals.actionLabel ?: "Dismiss")
                    }
                },
                containerColor = containerColor,
                contentColor = contentColor,
                actionContentColor = actionColor,
                shape = MaterialTheme.shapes.medium
            ) {
                Text(text = data.visuals.message)
            }
        }
    }
}