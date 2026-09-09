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
import kotlinx.coroutines.launch
import rustlib.my_yrs_lib.Block
import rustlib.my_yrs_lib.docFromSnapshot
import uniffi.protocol.Col
import uniffi.protocol.GetDataIn
import uniffi.protocol.SelectArgument
import z.zndroid.DbManager
import z.zndroid.Storage.StorageAccess
import z.zndroid.Storage.StorageKey
import z.zndroid.components.GlobalPopupManager

/**
 * A data class to hold the comparison between SQLite and CRDT versions of a block.
 */
data class BlockComparison(
    val yrsId: String,
    val sqliteContent: String?,
    val crdtContent: String?,
    val isMatch: Boolean
)

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AdminViewOfPage(
    pageId: String,
    onBack: () -> Unit
) {
    var comparisons by remember { mutableStateOf(emptyList<BlockComparison>()) }
    var isLoading by remember { mutableStateOf(true) }
    val coroutineScope = rememberCoroutineScope()

    LaunchedEffect(pageId) {
        isLoading = true
        try {
            // 1. Fetch CRDT blocks from the 'pages' blob
            val pageRow = DbManager.getPage(pageId).getOrThrow()
            val userId = StorageAccess.getUserId()
            val blob = (pageRow.cols.getOrNull(2) as? Col.Blob)?.v1
            
            val crdtBlocks = if (blob != null) {
                val boss = docFromSnapshot(blob, userId, pageId)
                val blocks = boss.getEntirePage()
                boss.destroy()
                blocks
            } else {
                emptyList<Block>()
            }

            // 2. Fetch SQLite blocks from 'every_block_in_existence'
            val sqliteData = DbManager.getData(GetDataIn(
                tableName = "every_block_in_existence",
                arguments = listOf(SelectArgument.XEqualY("id_of_page_i_belong_to", pageId)),
                columnsToRead = emptyList()
            )).getOrThrow()

            // 3. Compare them
            val sqliteMap = sqliteData.rows.associate { row ->
                // index 3 is my_id_as_given_by_yrs, index 2 is content
                val yrsId = (row.cols.getOrNull(3) as? Col.Text)?.v1 ?: ""
                val content = (row.cols.getOrNull(2) as? Col.Text)?.v1 ?: ""
                yrsId to content
            }

            val crdtMap = crdtBlocks.associate { it.idInYrs to it.text }

            // Merge IDs from both sources
            val allIds = (sqliteMap.keys + crdtMap.keys).distinct()

            comparisons = allIds.map { id ->
                val sText = sqliteMap[id]
                val cText = crdtMap[id]
                BlockComparison(
                    yrsId = id,
                    sqliteContent = sText,
                    crdtContent = cText,
                    isMatch = sText == cText
                )
            }
        } catch (e: Exception) {
            GlobalPopupManager.show("Admin View error: ${e.message}")
        } finally {
            isLoading = false
        }
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Admin: $pageId") },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Text("←")
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
    val bgColor = if (comp.isMatch) Color(0xFFE8F5E9) else Color(0xFFFFEBEE) // Subtle Green or Red
    val contentColor = if (comp.isMatch) Color(0xFF2E7D32) else Color(0xFFC62828)

    Card(
        modifier = Modifier.fillMaxWidth(),
        colors = CardDefaults.cardColors(containerColor = bgColor)
    ) {
        Column(modifier = Modifier.padding(12.dp)) {
            Text(
                text = "Yrs ID: ${comp.yrsId}",
                style = MaterialTheme.typography.labelSmall,
                color = contentColor.copy(alpha = 0.7f)
            )
            Spacer(modifier = Modifier.height(4.dp))
            
            Row(modifier = Modifier.fillMaxWidth()) {
                Column(modifier = Modifier.weight(1f)) {
                    Text("SQLite", style = MaterialTheme.typography.labelMedium, color = contentColor)
                    Text(comp.sqliteContent ?: "[NULL]", style = MaterialTheme.typography.bodySmall)
                }
                Column(modifier = Modifier.weight(1f)) {
                    Text("CRDT", style = MaterialTheme.typography.labelMedium, color = contentColor)
                    Text(comp.crdtContent ?: "[NULL]", style = MaterialTheme.typography.bodySmall)
                }
            }
            
            if (!comp.isMatch) {
                Text(
                    text = "Mismatch Detected!",
                    style = MaterialTheme.typography.labelLarge,
                    color = Color.Red,
                    modifier = Modifier.padding(top = 8.dp)
                )
            }
        }
    }
}
