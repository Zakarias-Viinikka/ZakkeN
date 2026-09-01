package z.zndroid

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import rustlib.client_table_blueprints.*
import uniffi.protocol.ColumnDef
import uniffi.protocol.ForeignKeyDef

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SchemaGui(onBack: () -> Unit) {
    val tableSchemas = listOf(
        TableSchema("pages", pagesColumns(), emptyList()),
        TableSchema("uncommitted_diffs", uncommittedDiffsColumns(), emptyList()),
        TableSchema("backlinks", backlinksColumns(), getForeignDefBacklinks()),
        // TODO: Add every_block_in_existence when it is exported by the library
    )

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Schema Viewer") },
                navigationIcon = {
                    Button(onClick = onBack) { Text("Back") }
                }
            )
        }
    ) { innerPadding ->
        LazyColumn(
            modifier = Modifier
                .padding(innerPadding)
                .fillMaxSize()
                .padding(16.dp)
        ) {
            items(tableSchemas) { schema ->
                SchemaCard(schema)
                Spacer(modifier = Modifier.height(16.dp))
            }
        }
    }
}

data class TableSchema(
    val name: String,
    val columns: List<ColumnDef>,
    val foreignKeys: List<ForeignKeyDef>
)

@Composable
fun SchemaCard(schema: TableSchema) {
    Card(modifier = Modifier.fillMaxWidth()) {
        Column(modifier = Modifier.padding(16.dp)) {
            Text(text = "Table: ${schema.name}", style = MaterialTheme.typography.headlineSmall)
            HorizontalDivider(modifier = Modifier.padding(vertical = 8.dp))
            
            Text(text = "Columns:", style = MaterialTheme.typography.titleMedium)
            schema.columns.forEach { col ->
                Column(modifier = Modifier.padding(start = 8.dp, top = 4.dp)) {
                    Text(
                        text = "• ${col.name}: ${col.columnType}",
                        style = MaterialTheme.typography.bodyMedium
                    )
                    if (col.primaryKey) Text("  [Primary Key]", style = MaterialTheme.typography.labelSmall, color = MaterialTheme.colorScheme.primary)
                    if (col.notNull) Text("  [Not Null]", style = MaterialTheme.typography.labelSmall)
                    if (col.unique) Text("  [Unique]", style = MaterialTheme.typography.labelSmall)
                    if (col.defaultValue.isNotEmpty()) Text("  Default: ${col.defaultValue}", style = MaterialTheme.typography.labelSmall)
                }
            }

            if (schema.foreignKeys.isNotEmpty()) {
                Spacer(modifier = Modifier.height(12.dp))
                Text(text = "Foreign Keys:", style = MaterialTheme.typography.titleMedium)
                schema.foreignKeys.forEach { fk ->
                    Text(
                        text = "• ${fk.column} → ${fk.referencedTable}(${fk.referencedColumn})",
                        modifier = Modifier.padding(start = 8.dp, top = 4.dp),
                        style = MaterialTheme.typography.bodyMedium
                    )
                }
            }
        }
    }
}
