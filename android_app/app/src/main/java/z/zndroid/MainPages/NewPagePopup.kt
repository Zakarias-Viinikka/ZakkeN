package z.zndroid.MainPages

import android.view.WindowManager
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import androidx.compose.ui.window.DialogWindowProvider
import kotlinx.coroutines.delay

@Composable
fun NewPagePopup(
    onDismiss: () -> Unit,
    onConfirm: (String) -> Unit
) {
    var pageName by remember { mutableStateOf("") }
    val focusRequester = remember { FocusRequester() }
    val keyboardController = LocalSoftwareKeyboardController.current
    val view = LocalView.current

    // The Compose Dialog window defaults to SOFT_INPUT_STATE_HIDDEN, so
    // the IME never shows even when the TextField has focus. This is the
    // known issue tracked as issuetracker.google.com/issues/204502668.
    // Reaching into the dialog's window and setting the soft input mode
    // to VISIBLE is the fix. ADJUST_RESIZE makes the dialog resize
    // around the keyboard instead of letting the IME cover it.
    LaunchedEffect(Unit) {
        (view.parent as? DialogWindowProvider)?.window?.let { window ->
            window.setSoftInputMode(
                WindowManager.LayoutParams.SOFT_INPUT_STATE_VISIBLE or
                    WindowManager.LayoutParams.SOFT_INPUT_ADJUST_RESIZE
            )
        }
        // Belt-and-braces: after the window mode is set, wait for the
        // dialog to compose, then focus and ask for the keyboard again.
        // Some devices need this even with the window mode set.
        delay(200)
        focusRequester.requestFocus()
        keyboardController?.show()
    }

    Dialog(
        onDismissRequest = onDismiss,
        properties = DialogProperties(usePlatformDefaultWidth = false)
    ) {
        Card(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp),
            shape = MaterialTheme.shapes.large
        ) {
            Column(
                modifier = Modifier.padding(16.dp),
                horizontalAlignment = Alignment.CenterHorizontally
            ) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text("New Page", style = MaterialTheme.typography.titleLarge)
                    IconButton(onClick = onDismiss) {
                        Text("✕")
                    }
                }

                Spacer(modifier = Modifier.height(16.dp))

                OutlinedTextField(
                    value = pageName,
                    onValueChange = { pageName = it },
                    label = { Text("Page Name") },
                    modifier = Modifier.fillMaxWidth().focusRequester(focusRequester),
                    singleLine = true
                )

                Spacer(modifier = Modifier.height(24.dp))

                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.End
                ) {
                    TextButton(onClick = onDismiss) {
                        Text("Cancel")
                    }
                    Button(
                        onClick = { onConfirm(pageName) },
                        enabled = pageName.isNotBlank()
                    ) {
                        Text("Create")
                    }
                }
            }
        }
    }
}