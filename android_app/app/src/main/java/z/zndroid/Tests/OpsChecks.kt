package z.zndroid.Tests

import rustlib.my_yrs_lib.PositionToInsert
import rustlib.my_yrs_lib.docFromSnapshot
import rustlib.text_diff.getDiff
import z.zndroid.DbManager
import z.zndroid.Storage.StorageAccess
import z.zndroid.actions.createNewPage
import z.zndroid.actions.disablePage
import z.zndroid.actions.moveBlockToAnotherPage
import z.zndroid.actions.renamePage
import z.zndroid.db.reads.getBlocksForPageOrdered
import z.zndroid.db.reads.getPageBlob
import z.zndroid.db.reads.isMainMenuPage
import z.zndroid.editor.PageSession
import z.zndroid.editor.PendingFocus

object OpsChecks {
    private suspend fun withScratchPage(name: String, body: suspend (pageId: String) -> Unit): String? {
        val created = createNewPage(name).getOrThrow()
        val menu = z.zndroid.db.reads.getMenuPages().getOrThrow()
        val row = menu.firstOrNull { it.title == name } ?: error("Created page '$name' not in menu")
        val pageId = row.pageId
        try {
            body(pageId)
        } finally {
            DbManager.hardDeletePage(pageId)
        }
        return pageId
    }

    private suspend fun openSession(pageId: String): PageSession {
        val mirror = getBlocksForPageOrdered(pageId).getOrThrow()
        val blob = getPageBlob(pageId).getOrThrow()
        val userId = StorageAccess.getUserId()
        val isMainMenu = isMainMenuPage(pageId)
        val boss = docFromSnapshot(blob, userId, pageId, System.currentTimeMillis().toString())
        return PageSession(boss, pageId, isMainMenu, mirror) { }
    }

    private fun sessionRowIds(s: PageSession): List<String> = s.rows.map { it.blockId }
    private suspend fun mirrorIds(pageId: String): List<String> =
        getBlocksForPageOrdered(pageId).getOrThrow().map { it.blockId }

    private fun verifyMirrorAndOrder(pageId: String, session: PageSession, problems: MutableList<String>, ctx: String) {
        val mirror = kotlinx.coroutines.runBlocking { mirrorIds(pageId) }
        val sessionIds = sessionRowIds(session)
        if (mirror != sessionIds) {
            problems.add("$ctx: session rows $sessionIds != mirror $mirror")
        }
    }

    suspend fun createAndRename(): DevCheckResult {
        val id = "ops.create_and_rename"
        val name = "Ops create and rename"
        return try {
            withScratchPage("ops_create_${System.currentTimeMillis()}") { pageId ->
                renamePage(pageId, "renamed_ops").getOrThrow()

                val mirrorAfter = getBlocksForPageOrdered(pageId).getOrThrow()
                val titleAfter = mirrorAfter.firstOrNull { it.isTitle }?.content ?: "<none>"
                if (titleAfter != "renamed_ops") {
                    throw Exception("rename failed. titleAfter='$titleAfter'")
                }
            }
            pass(id, name, "create + rename round-trip succeeded")
        } catch (e: Exception) {
            fail(id, name, "Exception: ${e.message}", e.stackTraceToString())
        }
    }

    suspend fun splitMerge(): DevCheckResult {
        val id = "ops.split_merge"
        val name = "Ops split then merge"
        return try {
            withScratchPage("ops_split_${System.currentTimeMillis()}") { pageId ->
                val session = openSession(pageId)
                try {
                    session.onInput(1, androidx.compose.ui.text.input.TextFieldValue("hello", androidx.compose.ui.text.TextRange(5)))
                    session.onEnter(1, 3)
                    session.closeAndFlush().getOrThrow()
                    val problems = mutableListOf<String>()
                    verifyMirrorAndOrder(pageId, session, problems, "after split")
                    if (session.rows.size != 3) problems.add("after split expected 3 rows, got ${session.rows.size}")
                    session.onBackspaceAtStart(2)
                    session.closeAndFlush().getOrThrow()
                    verifyMirrorAndOrder(pageId, session, problems, "after merge")
                    if (session.rows.size != 2) problems.add("after merge expected 2 rows, got ${session.rows.size}")
                    if (problems.isNotEmpty()) throw Exception(problems.joinToString("; "))
                } finally {
                    session.boss.destroy()
                }
            }
            pass(id, name, "split then merge round-trip matched mirror")
        } catch (e: Exception) {
            fail(id, name, "Exception: ${e.message}", e.stackTraceToString())
        }
    }

    suspend fun deleteRules(): DevCheckResult {
        val id = "ops.delete_rules"
        val name = "Ops delete rules (title, only-body protected)"
        return try {
            withScratchPage("ops_delete_${System.currentTimeMillis()}") { pageId ->
                val session = openSession(pageId)
                try {
                    val before = session.rows.size
                    session.onDeleteBlock(0)
                    if (session.rows.size != before) throw Exception("Title delete was allowed")
                    session.onDeleteBlock(1)
                    if (session.rows.size != before) throw Exception("Only-body delete was allowed")
                } finally {
                    session.boss.destroy()
                }
            }
            pass(id, name, "title and only-body deletes refused, rows unchanged")
        } catch (e: Exception) {
            fail(id, name, "Exception: ${e.message}", e.stackTraceToString())
        }
    }

    suspend fun moveSamePage(): DevCheckResult {
        val id = "ops.move_same_page"
        val name = "Ops move same page"
        return try {
            withScratchPage("ops_move_${System.currentTimeMillis()}") { pageId ->
                val session = openSession(pageId)
                try {
                    session.onInput(1, androidx.compose.ui.text.input.TextFieldValue("A", androidx.compose.ui.text.TextRange(1)))
                    session.onEnter(1, 1)
                    session.onInput(2, androidx.compose.ui.text.input.TextFieldValue("B", androidx.compose.ui.text.TextRange(1)))
                    session.closeAndFlush().getOrThrow()
                    val before = sessionRowIds(session)
                    session.onMoveUp(2)
                    session.closeAndFlush().getOrThrow()
                    val problems = mutableListOf<String>()
                    verifyMirrorAndOrder(pageId, session, problems, "after move up")
                    if (before == sessionRowIds(session)) problems.add("row order did not change")
                    if (problems.isNotEmpty()) throw Exception(problems.joinToString("; "))
                } finally {
                    session.boss.destroy()
                }
            }
            pass(id, name, "move up reordered rows and mirror agreed")
        } catch (e: Exception) {
            fail(id, name, "Exception: ${e.message}", e.stackTraceToString())
        }
    }

    suspend fun moveCrossPage(): DevCheckResult {
        val id = "ops.move_cross_page"
        val name = "Ops move cross page"
        return try {
            val fromName = "ops_from_${System.currentTimeMillis()}"
            withScratchPage(fromName) { fromPageId ->
                withScratchPage("ops_to_${System.currentTimeMillis()}") { toPageId ->
                    val fromSession = openSession(fromPageId)
                    val toSession = openSession(toPageId)
                    try {
                        fromSession.onInput(1, androidx.compose.ui.text.input.TextFieldValue("traveler", androidx.compose.ui.text.TextRange(8)))
                        fromSession.closeAndFlush().getOrThrow()
                        val movedBlockId = fromSession.rows[1].blockId
                        fromSession.onDeleteBlock(1)
                        fromSession.closeAndFlush().getOrThrow()
                        val moveResult = moveBlockToAnotherPage(fromSession.boss, toSession.boss, movedBlockId, toSession.rows.last().blockId, null)
                        moveResult.getOrThrow()
                    } finally {
                        fromSession.boss.destroy()
                        toSession.boss.destroy()
                    }
                }
            }
            pass(id, name, "cross-page move pushed a block to another page")
        } catch (e: Exception) {
            fail(id, name, "Exception: ${e.message}", e.stackTraceToString())
        }
    }

    suspend fun disableHidesFromMenu(): DevCheckResult {
        val id = "ops.disable_hides_from_menu"
        val name = "Ops disable hides from menu"
        return try {
            val scratch = "ops_disable_${System.currentTimeMillis()}"
            createNewPage(scratch).getOrThrow()
            val menu = z.zndroid.db.reads.getMenuPages().getOrThrow()
            val row = menu.firstOrNull { it.title == scratch } ?: throw Exception("Scratch page not in menu")
            val pageId = row.pageId
            try {
                disablePage(pageId).getOrThrow()
                val after = z.zndroid.db.reads.getMenuPages().getOrThrow()
                if (after.any { it.pageId == pageId }) {
                    throw Exception("Disabled page still in menu")
                }
            } finally {
                DbManager.hardDeletePage(pageId)
            }
            pass(id, name, "page disappeared from menu after disable")
        } catch (e: Exception) {
            fail(id, name, "Exception: ${e.message}", e.stackTraceToString())
        }
    }

    suspend fun unicodeEdit(): DevCheckResult {
        val id = "ops.unicode_edit"
        val name = "Ops unicode edit (accents, emoji, split mid-string)"
        return try {
            withScratchPage("ops_unicode_${System.currentTimeMillis()}") { pageId ->
                val session = openSession(pageId)
                try {
                    val text = "aåäö🙂end"
                    session.onInput(1, androidx.compose.ui.text.input.TextFieldValue(text, androidx.compose.ui.text.TextRange(text.length)))
                    session.onEnter(1, 4)
                    session.closeAndFlush().getOrThrow()
                    val mirror = mirrorIds(pageId)
                    if (mirror.size != 3) throw Exception("expected 3 mirror rows after unicode split, got ${mirror.size}")
                    val rows = getBlocksForPageOrdered(pageId).getOrThrow()
                    val joined = rows[1].content + rows[2].content
                    if (joined != text) throw Exception("after split, joined=$joined != original=$text")
                } finally {
                    session.boss.destroy()
                }
            }
            pass(id, name, "unicode split matched mirror")
        } catch (e: Exception) {
            fail(id, name, "Exception: ${e.message}", e.stackTraceToString())
        }
    }
}
