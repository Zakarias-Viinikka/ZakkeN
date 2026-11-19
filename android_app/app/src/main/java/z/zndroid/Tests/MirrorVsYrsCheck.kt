package z.zndroid.Tests

import android.util.Log
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
    private const val TAG = "MirrorVsYrs"

    suspend fun run(): DevCheckResult {
        try {
            DbManager.awaitReady()
        } catch (e: Exception) {
            return fail(ID, NAME, "DB not ready: ${e.message}", e.stackTraceToString())
        }
        return try {
            val userId = StorageAccess.getUserId()
            val pageIds = DbManager.getData(uniffi.protocol.GetDataIn(
                "pages",
                uniffi.protocol.SelectArguments.Single(uniffi.protocol.SelectArgument.All),
                listOf("page_id")
            )).getOrThrow().rows.mapNotNull {
                (it.cols.firstOrNull() as? uniffi.protocol.Col.Text)?.v1
            }

            val problems = mutableListOf<String>()
            Log.i(TAG, "checking ${pageIds.size} pages")

            for (pageId in pageIds) {
                val blob = getPageBlob(pageId).getOrThrow()
                val boss = docFromSnapshot(blob, userId, pageId, System.currentTimeMillis().toString())
                val yrsBlocks = try { boss.getEntirePage() } finally { boss.destroy() }
                val mirror = getBlocksForPageOrdered(pageId).getOrThrow()

                val yrsIds = yrsBlocks.map { it.idInYrs }
                val mirrorIds = mirror.map { it.blockId }
                val titleRows = mirror.filter { it.isTitle }
                val activeBlob = getActivePagesBlob(pageId).getOrThrow()
                val active = YrsActivePages(activeBlob)
                val activeFlag = active.isPageActive(pageId)
                val mirrorDisabled = isPageDisabled(pageId)
                Log.i(
                    TAG,
                    "page $pageId: yrs=${yrsBlocks.size} mirror=${mirror.size} " +
                        "titleRows=${titleRows.size} active=$activeFlag mirrorDisabled=$mirrorDisabled"
                )

                if (yrsBlocks.size != mirror.size) {
                    val p = "$pageId: yrs has ${yrsBlocks.size} blocks, mirror has ${mirror.size}. " +
                        "yrsIds=$yrsIds mirrorIds=$mirrorIds"
                    problems.add(p)
                    Log.w(TAG, p)
                    continue
                }

                if (titleRows.size != 1) {
                    val p = "$pageId: expected exactly 1 is_title row, found ${titleRows.size}"
                    problems.add(p)
                    Log.w(TAG, p)
                } else if (mirror.isEmpty() || !mirror.first().isTitle) {
                    val p = "$pageId: first mirror block is not the title"
                    problems.add(p)
                    Log.w(TAG, p)
                }

                if (yrsIds != mirrorIds) {
                    val p = "$pageId: order mismatch. yrs=$yrsIds mirror=$mirrorIds"
                    problems.add(p)
                    Log.w(TAG, p)
                } else {
                    yrsBlocks.zip(mirror).forEach { (y, m) ->
                        if (y.text != m.content) {
                            val p = "$pageId/${y.idInYrs}: content mismatch yrs='${y.text}' mirror='${m.content}'"
                            problems.add(p)
                            Log.w(TAG, p)
                        }
                    }
                }

                if (activeFlag == mirrorDisabled) {
                    val p = "$pageId: expected active=${!mirrorDisabled} but active_pages says active=$activeFlag " +
                        "(mirror page_is_disabled=$mirrorDisabled)"
                    problems.add(p)
                    Log.w(TAG, p)
                }
            }

            Log.i(TAG, "done: ${problems.size} problem(s) across ${pageIds.size} page(s)")

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