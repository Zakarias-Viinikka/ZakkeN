package z.zndroid.Tests

import rustlib.my_yrs_lib.YrsActivePages
import rustlib.my_yrs_lib.docFromSnapshot
import z.zndroid.DbManager
import z.zndroid.Storage.StorageAccess
import z.zndroid.db.reads.getActivePagesBlob
import z.zndroid.db.reads.getBlocksForPageOrdered
import z.zndroid.db.reads.getPageBlob
import z.zndroid.db.reads.isPageDisabled

object MirrorVsYrsCheck {
    const val ID = "mirror.vs_yrs"
    const val NAME = "Mirror vs yrs (every page)"

    suspend fun run(): DevCheckResult {
        if (DbManager.executeNative { }.isFailure) {
            return pass(ID, NAME, "Skipped (DB not ready)")
        }
        return try {
            val userId = StorageAccess.getUserId()
            val pagesOut = DbManager.getPages().getOrThrow()
            val pageIdIdx = 1 // pages: id, page_id, blobbed_page, page_status, version, is_main_menu_page
            val pageIds = pagesOut.mapNotNull { (it.cols.getOrNull(pageIdIdx) as? uniffi.protocol.Col.Text)?.v1 }

            val problems = mutableListOf<String>()

            for (pageId in pageIds) {
                val blob = getPageBlob(pageId).getOrThrow()
                val boss = docFromSnapshot(blob, userId, pageId, System.currentTimeMillis().toString())
                val yrsBlocks = try { boss.getEntirePage() } finally { boss.destroy() }
                val mirror = getBlocksForPageOrdered(pageId).getOrThrow()

                if (yrsBlocks.size != mirror.size) {
                    problems.add("$pageId: yrs has ${yrsBlocks.size} blocks, mirror has ${mirror.size}")
                    continue
                }

                if (mirror.size < 2) {
                    problems.add("$pageId: only ${mirror.size} block(s) in mirror (min is 2)")
                }

                val titleRows = mirror.filter { it.isTitle }
                if (titleRows.size != 1) {
                    problems.add("$pageId: expected exactly 1 is_title row, found ${titleRows.size}")
                } else if (!mirror.first().isTitle) {
                    problems.add("$pageId: first mirror block is not the title")
                }

                val yrsIds = yrsBlocks.map { it.idInYrs }
                val mirrorIds = mirror.map { it.blockId }
                if (yrsIds != mirrorIds) {
                    problems.add("$pageId: order mismatch. yrs=$yrsIds mirror=$mirrorIds")
                } else {
                    yrsBlocks.zip(mirror).forEach { (y, m) ->
                        if (y.text != m.content) {
                            problems.add("$pageId/${y.idInYrs}: content mismatch yrs='${y.text}' mirror='${m.content}'")
                        }
                    }
                }

                val activeBlob = getActivePagesBlob(pageId).getOrThrow()
                val active = YrsActivePages(activeBlob)
                val activeFlag = active.isPageActive(pageId)
                val mirrorDisabled = isPageDisabled(pageId)
                if (activeFlag == mirrorDisabled) {
                    problems.add("$pageId: active_pages says active=$activeFlag but mirror page_is_disabled=$mirrorDisabled")
                }
            }

            if (problems.isEmpty()) {
                pass(ID, NAME, "Checked ${pageIds.size} pages. All mirror rows agree with yrs.")
            } else {
                fail(ID, NAME, "${problems.size} problem(s) across ${pageIds.size} page(s).", problems.joinToString("\n"))
            }
        } catch (e: Exception) {
            fail(ID, NAME, "Exception: ${e.message}", e.stackTraceToString())
        }
    }
}
