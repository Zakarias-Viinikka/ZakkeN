package z.zndroid.Tests

import android.util.Log
import z.zndroid.DbManager
import z.zndroid.actions.createNewPage
import uniffi.protocol.*

/**
 * Dev-page actions. Not part of the app's normal flow. Each logs a DevAction
 * line so the e2e harness can see what was done.
 */
object Actions {
    private const val TAG = "DevAction"

    suspend fun wipeAllPageData(): DevCheckResult {
        return try {
            DbManager.withTransaction {
                // Children first, parent last: pages has foreign keys
                // pointing from the other three.
                listOf("every_block_in_existence", "client_love_letter", "pages").forEach { table ->
                    DbManager.deleteRowWhere(DeleteRowWhereIn(
                        table,
                        SelectArguments.Single(SelectArgument.All)
                    )).getOrThrow()
                }
            }.getOrThrow()
            Log.i(TAG, "wiped every_block_in_existence, client_love_letter, pages")
            pass("actions.wipe_all_page_data", "Wipe all page data", "Deleted all rows from 4 tables.")
        } catch (e: Exception) {
            fail("actions.wipe_all_page_data", "Wipe all page data", "Exception: ${e.message}", e.stackTraceToString())
        }
    }

    suspend fun seedThreePages(): DevCheckResult {
        val seeds = listOf("Grocery list", "Work notes", "Ideas")
        return try {
            for (name in seeds) {
                createNewPage(name).getOrThrow()
            }
            Log.i(TAG, "seeded 3 pages")
            pass("actions.seed_three_pages", "Seed three pages", "Created 3 pages via createNewPage.")
        } catch (e: Exception) {
            fail("actions.seed_three_pages", "Seed three pages", "Exception: ${e.message}", e.stackTraceToString())
        }
    }
}
