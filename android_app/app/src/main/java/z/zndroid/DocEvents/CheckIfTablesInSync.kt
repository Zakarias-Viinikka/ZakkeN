package z.zndroid.DocEvents

import rustlib.my_yrs_lib.docFromSnapshot
import uniffi.protocol.Col
import uniffi.protocol.GetDataIn
import uniffi.protocol.SelectArgument
import z.zndroid.DbManager
import z.zndroid.Storage.StorageAccess

/**
 * Result of comparing a single block between SQLite and CRDT.
 */
data class BlockComparison(
    val yrsId: String,
    val sqliteContent: String?,
    val crdtContent: String?,
    val isMatch: Boolean
)

/**
 * Logic to verify that the queryable SQLite tables match the authoritative CRDT state.
 */
object CheckIfTablesInSync {
    
    /**
     * Compares all blocks for a specific page between 'every_block_in_existence' and the 'pages' blob.
     */
    suspend fun checkPageSync(pageId: String): Result<List<BlockComparison>> {
        return try {
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
                emptyList()
            }

            // 2. Fetch SQLite blocks from 'every_block_in_existence'
            val sqliteData = DbManager.getData(GetDataIn(
                tableName = "every_block_in_existence",
                arguments = listOf(SelectArgument.XEqualY("id_of_page_i_belong_to", pageId, null)),
                columnsToRead = emptyList()
            )).getOrThrow()

            // 3. Resolve column indices dynamically
            val checkOut = DbManager.checkTable(uniffi.protocol.CheckTableIn("every_block_in_existence")).getOrThrow()
            val idIndex = checkOut.columns.indexOfFirst { it.name == "my_id_as_given_by_yrs" }
            val contentIndex = checkOut.columns.indexOfFirst { it.name == "content" }

            val sqliteMap = sqliteData.rows.associate { row ->
                val yrsId = (row.cols.getOrNull(idIndex) as? Col.Text)?.v1 ?: ""
                val content = (row.cols.getOrNull(contentIndex) as? Col.Text)?.v1 ?: ""
                yrsId to content
            }

            val crdtMap = crdtBlocks.associate { it.idInYrs to it.text }

            // 4. Merge IDs from both sources and compare
            val allIds = (sqliteMap.keys + crdtMap.keys).distinct()
            val results = allIds.map { id ->
                val sText = sqliteMap[id]
                val cText = crdtMap[id]
                BlockComparison(
                    yrsId = id,
                    sqliteContent = sText,
                    crdtContent = cText,
                    isMatch = sText == cText
                )
            }
            
            Result.success(results)
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
}
