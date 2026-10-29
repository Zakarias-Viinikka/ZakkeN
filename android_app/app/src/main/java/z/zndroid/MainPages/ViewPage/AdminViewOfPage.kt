package z.zndroid.MainPages.ViewPage

import androidx.compose.foundation.background
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.unit.dp
import z.zndroid.Tests.BlockComparison
import z.zndroid.Tests.CheckIfTablesInSync
import z.zndroid.components.GlobalPopupManager

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AdminViewOfPage(
    pageId: String,
    onBack: () -> Unit
) {
    var comparisons by remember { mutableStateOf(emptyList<BlockComparison>()) }
    var isLoading by remember { mutableStateOf(true) }

    LaunchedEffect(pageId) {
        isLoading = true
        CheckIfTablesInSync.checkPageSync(pageId).fold(
            onSuccess = { comparisons = it },
            onFailure = { GlobalPopupManager.show("Admin View error: ${it.message}") }
        )
        isLoading = false
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Admin: $pageId") },
                navigationIcon = {
                    IconButton(onClick = onBack) { Text("←") }
                }
            )
        }
    ) { innerPadding ->
        if (isLoading) {
            Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                CircularProgressIndicator()
            }
        } else {
            LazyColumn(
                modifier = Modifier
                    .padding(innerPadding)
                    .fillMaxSize()
                    .padding(16.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                item {
                    Text("Comparison of SQLite vs CRDT", style = MaterialTheme.typography.titleMedium)
                    Spacer(modifier = Modifier.height(16.dp))
                }
                items(comparisons) { comp ->
                    ComparisonCard(comp)
                }
            }
        }
    }
}

@Composable
fun ComparisonCard(comp: BlockComparison) {
    var showOptions by remember { mutableStateOf(false) }
    val clipboardManager = LocalClipboardManager.current
    val containerColor = if (comp.isMatch) MaterialTheme.colorScheme.surface else MaterialTheme.colorScheme.errorContainer
    val statusColor = if (comp.isMatch) Color(0xFF4CAF50) else MaterialTheme.colorScheme.error

    Card(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 4.dp)
            .combinedClickable(
                onClick = { },
                onLongClick = { showOptions = true }
            ),
        colors = CardDefaults.cardColors(containerColor = containerColor),
        elevation = CardDefaults.cardElevation(defaultElevation = 1.dp)
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text(
                    text = "Block: ${comp.yrsId.take(8)}...",
                    style = MaterialTheme.typography.labelMedium,
                    color = MaterialTheme.colorScheme.outline
                )
                Surface(
                    color = statusColor.copy(alpha = 0.1f),
                    shape = androidx.compose.foundation.shape.CircleShape,
                    border = androidx.compose.foundation.BorderStroke(1.dp, statusColor.copy(alpha = 0.5f))
                ) {
                    Text(
                        text = if (comp.isMatch) "SYNCED" else "MISMATCH",
                        modifier = Modifier.padding(horizontal = 8.dp, vertical = 2.dp),
                        style = MaterialTheme.typography.labelSmall,
                        color = statusColor
                    )
                }
            }
            Spacer(modifier = Modifier.height(12.dp))
            Row(modifier = Modifier.fillMaxWidth()) {
                Column(modifier = Modifier.weight(1f)) {
                    Text("SQLite Table", style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.secondary)
                    Text(comp.sqliteContent ?: "[NULL]", style = MaterialTheme.typography.bodyMedium, modifier = Modifier.padding(top = 4.dp))
                }
                Box(
                    modifier = Modifier
                        .width(1.dp)
                        .height(40.dp)
                        .padding(horizontal = 8.dp)
                        .background(Color.Gray.copy(alpha = 0.3f))
                )
                Column(modifier = Modifier.weight(1f)) {
                    Text("Yrs CRDT", style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.secondary)
                    Text(comp.crdtContent ?: "[NULL]", style = MaterialTheme.typography.bodyMedium, modifier = Modifier.padding(top = 4.dp))
                }
            }
        }
    }

    if (showOptions) {
        AlertDialog(
            onDismissRequest = { showOptions = false },
            title = { Text("Block Options") },
            text = { Text("Copy content to clipboard for manual recovery?") },
            confirmButton = {
                TextButton(onClick = {
                    comp.sqliteContent?.let { clipboardManager.setText(AnnotatedString(it)) }
                    showOptions = false
                }) { Text("Copy SQLite") }
            },
            dismissButton = {
                TextButton(onClick = {
                    comp.crdtContent?.let { clipboardManager.setText(AnnotatedString(it)) }
                    showOptions = false
                }) { Text("Copy CRDT") }
            }
        )
    }
}