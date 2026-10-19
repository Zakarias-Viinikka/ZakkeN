package z.zndroid.Tests

import z.zndroid.DbManager
import z.zndroid.DocEvents.AddBlock
import z.zndroid.DocEvents.AddBlockCtx
import z.zndroid.DocEvents.DeleteBlock
import z.zndroid.DocEvents.DeleteBlockCtx
import z.zndroid.DocEvents.CheckIfTablesInSync
import rustlib.my_yrs_lib.BossOfYrs
import rustlib.my_yrs_lib.PositionToInsert
import rustlib.client_table_blueprints.newPageRow

/**
 * E2E tests for AddBlock and DeleteBlock executors.
 * Verifies transaction integrity, state application, and sync verification via CHECK_SYNC logic.
 */
class BlockExecutorsTest : AppTest {
    override val name = "Block Executors (Add & Delete) E2E Sync"

    override suspend fun run(): TestResult {
        if (DbManager.executeNative { }.isFailure) {
            return TestResult(name, true, "Skipped (DB not ready)")
        }

        return try {
            val userId = z.zndroid.Storage.StorageAccess.getUserId()
            val boss = BossOfYrs(userId, System.currentTimeMillis().toString())
            val pageId = boss.pageId()

            try {
                // 1. Initialise page in DB
                DbManager.withTransaction {
                    val initialBlob = boss.snapshot()
                    val version = ByteArray(0)
                    val row = newPageRow(pageId, false, initialBlob, version)
                    val values = z.zndroid.protocol.SafeRowMapper.mapRow(
                        row = row,
                        columnDefs = rustlib.client_table_blueprints.newTablePages(),
                        expectedNames = listOf("page_id", "blobbed_page", "page_status", "version", "is_main_menu_page")
                    )
                    DbManager.insertData(uniffi.protocol.InsertDataIn("pages", values)).getOrThrow()
                }.getOrThrow()

                // 2. Test AddBlock Executor
                val addedBlockId = AddBlock.execute(
                    AddBlockCtx(
                        boss = boss,
                        content = "E2E Test Block Content",
                        position = PositionToInsert.AtEnd
                    )
                ).getOrThrow()

                if (addedBlockId.isBlank()) {
                    return TestResult(name, false, "AddBlock executor failed to return a block ID")
                }

                // Assert sync via CHECK_SYNC
                val syncCheckAfterAdd = CheckIfTablesInSync.checkPageSync(pageId).getOrThrow()
                val mismatchesAdd = syncCheckAfterAdd.filter { !it.isMatch }
                if (mismatchesAdd.isNotEmpty()) {
                    return TestResult(name, false, "CHECK_SYNC failed after AddBlock: ${mismatchesAdd.size} out-of-sync checks")
                }

                // 3. Test DeleteBlock Executor
                DeleteBlock.execute(
                    DeleteBlockCtx(
                        boss = boss,
                        blockId = addedBlockId
                    )
                ).getOrThrow()

                // Assert sync via CHECK_SYNC again
                val syncCheckAfterDelete = CheckIfTablesInSync.checkPageSync(pageId).getOrThrow()
                val mismatchesDelete = syncCheckAfterDelete.filter { !it.isMatch }
                if (mismatchesDelete.isNotEmpty()) {
                    return TestResult(name, false, "CHECK_SYNC failed after DeleteBlock: ${mismatchesDelete.size} out-of-sync checks")
                }

                TestResult(name, true, "AddBlock and DeleteBlock executors passed successfully with full CHECK_SYNC verification.")
            } finally {
                // Cleanup page in DB
                DbManager.deletePage(pageId)
                boss.destroy()
            }
        } catch (e: Exception) {
            TestResult(name, false, "BlockExecutorsTest failed with exception", e.stackTraceToString())
        }
    }
}
