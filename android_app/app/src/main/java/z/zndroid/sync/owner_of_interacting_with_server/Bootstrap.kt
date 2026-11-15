package z.zndroid.sync.owner_of_interacting_with_server

import kotlinx.coroutines.flow.update
import rustlib.love_letter.LoveLetter
import rustlib.my_yrs_lib.YrsActivePages
import rustlib.server_link.PageSnapshot
import rustlib.server_link.ServerLink
import uniffi.protocol.GetDataIn
import uniffi.protocol.SelectArgument
import uniffi.protocol.SelectArguments
import z.zndroid.DbManager
import z.zndroid.db.schema.PagesTableSchema
import java.util.concurrent.atomic.AtomicBoolean

private val attempted = AtomicBoolean(false)

/** After a local wipe the next login must pull every page again. */
internal fun resetStartOverFresh() {
    attempted.set(false)
}

/**
 * Mirrors web's start_over_fresh_guard. A client with no local pages
 * pulls every page from the server once per process and applies each as
 * a CreateNewPage (plus DisablePage when the page is disabled). Throws on
 * failure; the flag is only set after every page applied, so the next
 * login tries again.
 */
internal suspend fun startOverFreshGuard(link: ServerLink) {
    if (attempted.get()) return
    if (!syncMayRun()) return
    if (localDbHasPages()) {
        attempted.set(true)
        return
    }
    val pages = link.getAllPages()
    for (page in pages) applyBootstrapPage(page)
    attempted.set(true)
    menuTickFlow.update { it + 1 }
}

private suspend fun localDbHasPages(): Boolean {
    val out = DbManager.getData(
        GetDataIn(
            PagesTableSchema.TABLE_NAME,
            SelectArguments.Single(SelectArgument.All),
            listOf(PagesTableSchema.pageId.name)
        )
    ).getOrThrow()
    return out.rows.isNotEmpty()
}

internal suspend fun applyBootstrapPage(page: PageSnapshot) {
    val created = applyLetter(
        LoveLetter.CreateNewPage(
            pageId = page.pageId,
            isMainMenuPage = page.isMainMenuPage,
            snapshotOfEdit = page.blobbedPage
        )
    )
    if (created is ApplyResult.Gated) {
        throw IllegalStateException("bootstrap: gated while applying page ${page.pageId}")
    }
    if (!pageIsActive(page)) {
        val disabled = applyLetter(
            LoveLetter.DisablePage(
                pageId = page.pageId,
                snapshotOfEdit = page.pageStatus
            )
        )
        if (disabled is ApplyResult.Gated) {
            throw IllegalStateException("bootstrap: gated while disabling page ${page.pageId}")
        }
    }
}

/** An empty status blob means the page was never disabled. */
private fun pageIsActive(page: PageSnapshot): Boolean {
    if (page.pageStatus.isEmpty()) return true
    val active = YrsActivePages(page.pageStatus)
    try {
        return active.isPageActive(page.pageId)
    } finally {
        active.destroy()
    }
}
