package z.zndroid.MainPages

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import z.zndroid.BuildConfig
import z.zndroid.components.GlobalPopupManager
import z.zndroid.sync.owner_of_interacting_with_server.ConnState
import z.zndroid.sync.owner_of_interacting_with_server.ReconcileReady
import z.zndroid.sync.owner_of_interacting_with_server.connState
import z.zndroid.sync.owner_of_interacting_with_server.currentLink
import z.zndroid.sync.owner_of_interacting_with_server.finishReconcileServerWins
import z.zndroid.sync.owner_of_interacting_with_server.loadAndClassify
import z.zndroid.sync.owner_of_interacting_with_server.wipeLocalAndAdoptServer
import z.zndroid.sync.owner_of_interacting_with_server.wipeServerAndPushMine

private enum class RecoveryAction { Reconcile, WipeSelf, WipeServer }

private fun confirmText(action: RecoveryAction, ready: ReconcileReady?): String = when (action) {
    RecoveryAction.Reconcile -> {
        val m = ready?.mismatched?.size ?: 0
        val c = ready?.clientOnly?.size ?: 0
        "$m differing page(s) will be replaced by the server's version; local edits to them are lost. " +
            "$c page(s) only on this device will be added to the server. Unsent edits are discarded."
    }
    RecoveryAction.WipeSelf ->
        "Every page and unsent edit on this device is deleted, then everything is pulled from the server."
    RecoveryAction.WipeServer ->
        "Every page on the server is deleted and replaced with this device's pages. Other devices will go out of sync."
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun RecoveryPage(onBack: () -> Unit) {
    val conn by connState.collectAsState()
    val scope = rememberCoroutineScope()
    var preview by remember { mutableStateOf<ReconcileReady?>(null) }
    var previewError by remember { mutableStateOf<String?>(null) }
    var busy by remember { mutableStateOf(false) }
    var confirming by remember { mutableStateOf<RecoveryAction?>(null) }

    val startedInSync = remember { connState.value == ConnState.Connected }

    BackHandler(enabled = busy) {}

    suspend fun loadPreview() {
        previewError = null
        val link = currentLink()
        if (link == null) {
            previewError = "no server link"
            return
        }
        try {
            preview = loadAndClassify(link)
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            preview = null
            previewError = e.message ?: e.toString()
        }
    }

    LaunchedEffect(conn) {
        when (conn) {
            ConnState.Connected -> if (!startedInSync) onBack()
            ConnState.Lonewolf -> loadPreview()
            else -> preview = null
        }
    }

    fun run(action: RecoveryAction) {
        if (busy) return
        busy = true
        scope.launch {
            try {
                when (action) {
                    RecoveryAction.Reconcile -> {
                        val link = currentLink() ?: throw IllegalStateException("not connected")
                        val ready = preview ?: throw IllegalStateException("nothing loaded yet")
                        finishReconcileServerWins(link, ready)
                    }
                    RecoveryAction.WipeSelf -> wipeLocalAndAdoptServer()
                    RecoveryAction.WipeServer -> {
                        val link = currentLink() ?: throw IllegalStateException("not connected")
                        wipeServerAndPushMine(link)
                    }
                }
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                GlobalPopupManager.showError("Recovery failed: ${e.message ?: e}")
                if (connState.value == ConnState.Lonewolf) loadPreview()
            } finally {
                busy = false
            }
        }
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Out of sync") },
                navigationIcon = {
                    IconButton(onClick = { if (!busy) onBack() }) { Text("←") }
                }
            )
        }
    ) { innerPadding ->
        Column(
            modifier = Modifier
                .padding(innerPadding)
                .fillMaxSize()
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Text("This device and the server no longer share a dataset. Sync is paused until you pick one of these.")

            val ready = preview
            when {
                conn == ConnState.Connected -> Text("Connected and in sync. Debug access to the recovery actions.")
                conn == ConnState.LonewolfAndOffline -> Text("Waiting for the server connection...")
                previewError != null -> Text("Could not read the server's state: $previewError")
                ready == null -> Text("Reading pages and asking the server...")
                else -> Text(
                    "${ready.mismatched.size} page(s) differ from the server, " +
                        "${ready.clientOnly.size} page(s) exist only on this device."
                )
            }

            Button(
                onClick = { confirming = RecoveryAction.Reconcile },
                enabled = !busy && preview != null,
                modifier = Modifier.fillMaxWidth()
            ) { Text("Reconcile (server wins)") }
            Text(
                "Pages only on this device are added to the server. Pages that differ are replaced by the server's version.",
                style = MaterialTheme.typography.bodySmall
            )

            Button(
                onClick = { confirming = RecoveryAction.WipeSelf },
                enabled = !busy,
                modifier = Modifier.fillMaxWidth()
            ) { Text("Wipe this device and adopt server") }
            Text(
                "Deletes everything on this device and pulls the server's data.",
                style = MaterialTheme.typography.bodySmall
            )

            if (BuildConfig.DEBUG) {
                Button(
                    onClick = { confirming = RecoveryAction.WipeServer },
                    enabled = !busy && (conn == ConnState.Lonewolf || conn == ConnState.Connected),
                    modifier = Modifier.fillMaxWidth()
                ) { Text("Wipe server (mine wins)") }
                Text(
                    "Debug only. Deletes the server's data and uploads this device's pages.",
                    style = MaterialTheme.typography.bodySmall
                )
            }

            if (busy) CircularProgressIndicator()
        }
    }

    val pending = confirming
    if (pending != null) {
        AlertDialog(
            onDismissRequest = { confirming = null },
            title = {
                Text(
                    when (pending) {
                        RecoveryAction.Reconcile -> "Reconcile?"
                        RecoveryAction.WipeSelf -> "Wipe this device?"
                        RecoveryAction.WipeServer -> "Wipe the server?"
                    }
                )
            },
            text = { Text(confirmText(pending, preview)) },
            confirmButton = {
                TextButton(onClick = {
                    confirming = null
                    run(pending)
                }) { Text("Continue") }
            },
            dismissButton = {
                TextButton(onClick = { confirming = null }) { Text("Cancel") }
            }
        )
    }
}
