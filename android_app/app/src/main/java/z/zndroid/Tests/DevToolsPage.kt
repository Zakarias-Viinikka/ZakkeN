package z.zndroid.Tests

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.testTagsAsResourceId
import androidx.compose.ui.unit.dp
import android.util.Log
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DevToolsPage(onBack: () -> Unit) {
    val results = remember { mutableStateMapOf<String, DevCheckResult>() }
    var isRunning by remember { mutableStateOf(false) }
    val scope = rememberCoroutineScope()

    fun launchCheck(id: String) {
        if (isRunning) return
        isRunning = true
        scope.launch(Dispatchers.IO) {
            val runningResult = DevCheckResult(id, DevCheckRegistry.checks.first { it.id == id }.name, DevCheckStatus.RUNNING)
            results[id] = runningResult
            val result = DevCheckRegistry.runOne(id)
            results[id] = result
            isRunning = false
        }
    }

    fun runAll() {
        if (isRunning) return
        isRunning = true
        scope.launch(Dispatchers.IO) {
            var passCount = 0
            var failCount = 0
            DevCheckRegistry.checks.forEach { check ->
                results[check.id] = DevCheckResult(check.id, check.name, DevCheckStatus.RUNNING)
                val result = DevCheckRegistry.runOne(check.id)
                results[check.id] = result
                if (result.status == DevCheckStatus.PASS) passCount++ else if (result.status == DevCheckStatus.FAIL) failCount++
            }
            Log.i("DevCheck", "summary pass=$passCount fail=$failCount")
            isRunning = false
        }
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Dev Tools") },
                navigationIcon = { IconButton(onClick = onBack) { Text("←") } },
                actions = {
                    TextButton(onClick = { runAll() }, enabled = !isRunning) { Text("Run all") }
                    TextButton(onClick = { results.clear() }) { Text("Clear") }
                }
            )
        }
    ) { innerPadding ->
        Column(
            modifier = Modifier
                .padding(innerPadding)
                .fillMaxSize()
                .padding(16.dp)
                .semantics { testTagsAsResourceId = true }
        ) {
            val pass = results.values.count { it.status == DevCheckStatus.PASS }
            val fail = results.values.count { it.status == DevCheckStatus.FAIL }
            val running = results.values.count { it.status == DevCheckStatus.RUNNING }
            Text(
                text = "SUMMARY pass=$pass fail=$fail running=$running",
                style = MaterialTheme.typography.titleSmall,
                modifier = Modifier.testTag("dev_summary").padding(bottom = 8.dp)
            )
            TextButton(
                onClick = { runAll() },
                enabled = !isRunning,
                modifier = Modifier.testTag("dev_run_all").fillMaxWidth().padding(bottom = 8.dp)
            ) { Text("Run all checks") }
            LazyColumn(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                items(DevCheckRegistry.checks) { check ->
                    val res = results[check.id] ?: DevCheckResult(check.id, check.name, DevCheckStatus.NOT_RUN)
                    DevCheckCard(res, enabled = !isRunning) { launchCheck(check.id) }
                }
                item {
                    HorizontalDivider(modifier = Modifier.padding(vertical = 16.dp))
                    Text(
                        "Actions",
                        style = MaterialTheme.typography.titleSmall,
                        modifier = Modifier.padding(bottom = 8.dp)
                    )
                }
                item {
                    TextButton(
                        onClick = {
                            if (!isRunning) {
                                scope.launch(Dispatchers.IO) { Actions.wipeAllPageData() }
                            }
                        },
                        enabled = !isRunning,
                        modifier = Modifier.testTag("action_wipe").fillMaxWidth()
                    ) { Text("Wipe all page data") }
                }
                item {
                    TextButton(
                        onClick = {
                            if (!isRunning) {
                                scope.launch(Dispatchers.IO) { Actions.seedThreePages() }
                            }
                        },
                        enabled = !isRunning,
                        modifier = Modifier.testTag("action_seed").fillMaxWidth()
                    ) { Text("Seed three pages") }
                }
            }
        }
    }
}

@Composable
private fun DevCheckCard(result: DevCheckResult, enabled: Boolean, onClick: () -> Unit) {
    var expanded by remember { mutableStateOf(false) }
    val color = when (result.status) {
        DevCheckStatus.PASS -> Color(0xFF4CAF50)
        DevCheckStatus.FAIL -> MaterialTheme.colorScheme.error
        DevCheckStatus.RUNNING -> Color(0xFFAA8800)
        DevCheckStatus.NOT_RUN -> MaterialTheme.colorScheme.outline
    }
    Card(
        modifier = Modifier.fillMaxWidth().testTag("check_${result.id}"),
        onClick = { if (enabled) onClick() else expanded = !expanded },
        colors = CardDefaults.cardColors(containerColor = color.copy(alpha = 0.1f)),
        border = androidx.compose.foundation.BorderStroke(1.dp, color.copy(alpha = 0.5f))
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    text = "${result.name}: ${result.status}",
                    style = MaterialTheme.typography.titleMedium,
                    color = color
                )
                Text(
                    text = result.id,
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.outline
                )
            }
            if (result.message.isNotEmpty()) {
                Text(
                    text = result.message,
                    style = MaterialTheme.typography.bodyMedium,
                    modifier = Modifier.padding(top = 4.dp)
                )
            }
            if (expanded && result.detail != null) {
                HorizontalDivider(modifier = Modifier.padding(vertical = 8.dp))
                Text(
                    text = result.detail,
                    style = MaterialTheme.typography.bodySmall,
                    fontFamily = androidx.compose.ui.text.font.FontFamily.Monospace
                )
            }
        }
    }
}
