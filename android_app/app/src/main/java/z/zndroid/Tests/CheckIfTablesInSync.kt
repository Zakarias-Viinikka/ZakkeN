package z.zndroid.Tests

import rustlib.my_yrs_lib.docFromSnapshot
import uniffi.protocol.Col
import uniffi.protocol.GetDataIn
import uniffi.protocol.SelectArgument
import uniffi.protocol.SelectArguments
import z.zndroid.DbManager
import z.zndroid.Storage.StorageAccess

data class BlockComparison(
    val yrsId: String,
    val sqliteContent: String?,
    val crdtContent: String?,
    val isMatch: Boolean
)

object CheckIfTablesInSync {
    suspend fun checkPageSync(pageId: String): Result<List<BlockComparison>> {
        return try {
            val pageRow = DbManager.getPage(pageId).getOrThrow()
            val userId = StorageAccess.getUserId()
            val blob = (pageRow.cols.getOrNull(2) as? Col.Blob)?.v1

            val crdtBlocks = if (blob != null) {
                val boss = docFromSnapshot(blob, userId, pageId, System.currentTimeMillis().toString())
                val blocks = boss.getEntirePage()
                boss.destroy()
                blocks
            } else {
                emptyList()
            }

            val sqliteData = DbManager.getData(GetDataIn(
                tableName = "every_block_in_existence",
                arguments = SelectArguments.Single(SelectArgument.XEqualY("id_of_page_i_belong_to", pageId)),
                columnsToRead = emptyList()
            )).getOrThrow()

            val checkOut = DbManager.checkTable(uniffi.protocol.CheckTableIn("every_block_in_existence")).getOrThrow()
            val idIndex = checkOut.columns.indexOfFirst { it.name == "my_id_as_given_by_yrs" }
            val contentIndex = checkOut.columns.indexOfFirst { it.name == "content" }

            val sqliteMap = sqliteData.rows.associate { row ->
                val yrsId = (row.cols.getOrNull(idIndex) as? Col.Text)?.v1 ?: ""
                val content = (row.cols.getOrNull(contentIndex) as? Col.Text)?.v1 ?: ""
                yrsId to content
            }

            val crdtMap = crdtBlocks.associate { it.idInYrs to it.text }
            val allIds = (sqliteMap.keys + crdtMap.keys).distinct()
            val results = allIds.map { id ->
                val sText = sqliteMap[id]
                val cText = crdtMap[id]
                BlockComparison(id, sText, cText, sText == cText)
            }
            Result.success(results)
        } catch (e: Exception) {
            Result.failure(e)
        }
    }
}
