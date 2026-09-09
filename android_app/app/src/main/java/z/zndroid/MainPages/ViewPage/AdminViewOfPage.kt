package z.zndroid.MainPages.ViewPage

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import rustlib.my_yrs_lib.Block
import rustlib.my_yrs_lib.docFromSnapshot
import uniffi.protocol.Col
import uniffi.protocol.GetDataIn
import uniffi.protocol.SelectArgument
import z.zndroid.DbManager
import z.zndroid.DocEvents.BlockComparison
import z.zndroid.DocEvents.CheckIfTablesInSync
import z.zndroid.Storage.StorageAccess
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
                    IconButton(onClick = onBack) {
                        Text("←")
                    }
                },
                actions = {
                    IconButton(onClick = {
                        android.util.Log.d("ADMIN_DEBUG", "--- Admin Debug Info for Page: $pageId ---")
                        comparisons.forEach { comp ->
                            android.util.Log.d("ADMIN_DEBUG", "Block ID: ${comp.yrsId}")
                            android.util.Log.d("ADMIN_DEBUG", "  SQLite: [${comp.sqliteContent}]")
                            android.util.Log.d("ADMIN_DEBUG", "  CRDT:   [${comp.crdtContent}]")
                            android.util.Log.d("ADMIN_DEBUG", "  Match:  ${comp.isMatch}")
                        }
                        android.util.Log.d("ADMIN_DEBUG", "--- End Debug Info ---")
                    }) {
                        Text("Log")
                    }
                }
            )
        }
    ) { innerPadding ->
        if (isLoading) {
            Box(modifier = Modifier.fillMaxSize(), contentAlignment = androidx.compose.ui.Alignment.Center) {
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
    // Use Material 3 error colors for mismatches, and standard surface for matches
    val containerColor = if (comp.isMatch) {
        MaterialTheme.colorScheme.surface
    } else {
        MaterialTheme.colorScheme.errorContainer
    }

    val statusColor = if (comp.isMatch) {
        Color(0xFF4CAF50) // Material Green 500
    } else {
        MaterialTheme.colorScheme.error
    }

    Card(
        modifier = Modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(containerColor = containerColor),
        border = if (!comp.isMatch) CardDefaults.outlinedCardBorder() else null,
        elevation = CardDefaults.cardElevation(defaultElevation = 1.dp)
    ) {
        Column(modifier = Modifier.padding(16.dp)) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = androidx.compose.ui.Alignment.CenterVertically
            ) {
                Text(
                    text = "Block: ${comp.yrsId.take(8)}...",
                    style = MaterialTheme.typography.labelMedium,
                    color = MaterialTheme.colorScheme.outline
                )
                
                // Status badge
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
                // SQLite Column
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        "SQLite Table",
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.secondary
                    )
                    Text(
                        text = comp.sqliteContent ?: "[NULL]",
                        style = MaterialTheme.typography.bodyMedium,
                        modifier = Modifier.padding(top = 4.dp)
                    )
                }

                // Divider line
                Box(
                    modifier = Modifier
                        .width(1.dp)
                        .height(40.dp)
                        .padding(horizontal = 8.dp)
                        .background(MaterialTheme.colorScheme.outlineVariant)
                )

                // CRDT Column
                Column(modifier = Modifier.weight(1f)) {
                    Text(
                        "Yrs CRDT",
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.secondary
                    )
                    Text(
                        text = comp.crdtContent ?: "[NULL]",
                        style = MaterialTheme.typography.bodyMedium,
                        modifier = Modifier.padding(top = 4.dp)
                    )
                }
            }
        }
    }
}
