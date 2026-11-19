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
    onConfirm: (String) -> Unit,
    errorMessage: String? = null,
    isSubmitting: Boolean = false,
    onClearError: () -> Unit = {}
) {
    var pageName by remember { mutableStateOf("") }
    val focusRequester = remember { FocusRequester() }

    Dialog(
        onDismissRequest = onDismiss,
        properties = DialogProperties(usePlatformDefaultWidth = false)
    ) {
        // LocalView.current must be read inside the Dialog content so its
        // parent is the dialog's window, not the activity content view.
        val view = LocalView.current
        val keyboardController = LocalSoftwareKeyboardController.current

        LaunchedEffect(Unit) {
            (view.parent as? DialogWindowProvider)?.window?.let { window ->
                window.setSoftInputMode(
                    WindowManager.LayoutParams.SOFT_INPUT_STATE_VISIBLE or
                        WindowManager.LayoutParams.SOFT_INPUT_ADJUST_RESIZE
                )
            }
            delay(200)
            focusRequester.requestFocus()
            keyboardController?.show()
        }

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
                    IconButton(onClick = onDismiss, enabled = !isSubmitting) {
                        Text("✕")
                    }
                }

                Spacer(modifier = Modifier.height(16.dp))

                OutlinedTextField(
                    value = pageName,
                    onValueChange = {
                        pageName = it
                        if (errorMessage != null) onClearError()
                    },
                    label = { Text("Page Name") },
                    isError = errorMessage != null,
                    enabled = !isSubmitting,
                    modifier = Modifier.fillMaxWidth().focusRequester(focusRequester),
                    singleLine = true
                )

                if (errorMessage != null) {
                    Spacer(modifier = Modifier.height(8.dp))
                    Text(
                        text = errorMessage,
                        color = MaterialTheme.colorScheme.error,
                        style = MaterialTheme.typography.bodySmall,
                        modifier = Modifier.fillMaxWidth()
                    )
                }

                Spacer(modifier = Modifier.height(24.dp))

                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.End
                ) {
                    TextButton(onClick = onDismiss, enabled = !isSubmitting) {
                        Text("Cancel")
                    }
                    Button(
                        onClick = { onConfirm(pageName.trim()) },
                        enabled = pageName.isNotBlank() && !isSubmitting
                    ) {
                        Text("Create")
                    }
                }
            }
        }
    }
}