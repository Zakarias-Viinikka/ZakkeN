package z.zndroid.Tests

import uniffi.protocol.Col
import z.zndroid.DbManager
import z.zndroid.DocEvents.CheckIfTablesInSync

/**
 * Programmatically verifies that ALL pages in the database are correctly
 * synchronized between their binary CRDT blobs and queryable SQLite tables.
 */
class PageDataSyncTest : AppTest {
    override val name = "Page Data Integrity (Sync Check)"

    override suspend fun run(): TestResult {
        // If DB isn't ready, skip
        if (DbManager.executeNative { }.isFailure) {
            return TestResult(name, true, "Skipped (DB not ready)")
        }

        return try {
            val pages = DbManager.getPages().getOrThrow()
            val failures = mutableListOf<String>()

            pages.forEach { row ->
                // page_id is index 1 (id:0, page_id:1)
                val pageId = (row.cols.getOrNull(1) as? Col.Text)?.v1 ?: "unknown"
                
                CheckIfTablesInSync.checkPageSync(pageId).fold(
                    onSuccess = { comparisons ->
                        val mismatches = comparisons.filter { !it.isMatch }
                        if (mismatches.isNotEmpty()) {
                            failures.add("Page '$pageId' has ${mismatches.size} out-of-sync blocks.")
                        }
                    },
                    onFailure = { error ->
                        failures.add("Page '$pageId' failed sync check: ${error.message}")
                    }
                )
            }

            if (failures.isEmpty()) {
                TestResult(name, true, "Checked ${pages.size} pages. All SQLite data matches CRDT authoritative state.")
            } else {
                TestResult(name, false, "Integrity issues found in ${failures.size} pages", failures.joinToString("\n"))
            }
        } catch (e: Exception) {
            TestResult(name, false, "Test failed with exception", e.message)
        }
    }
}
