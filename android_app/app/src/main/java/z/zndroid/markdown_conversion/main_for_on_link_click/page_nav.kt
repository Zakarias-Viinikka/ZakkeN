package z.zndroid.markdown_conversion.main_for_on_link_click

//! Command [[PageNav|page id]]: open that page. `rest` is the page id.
//! The navigation callback is supplied by the view (ViewPage wires onOpenPage
//! through EditableBlock). No global router.

fun runPageNav(rest: String, onOpenPage: (String) -> Unit): Result<Unit> {
    val pageId = rest.trim()
    if (pageId.isEmpty()) {
        return Result.failure(
            OnLinkClickError(
                errMsg = "PageNav needs a page id after the |, got nothing",
                method = "run_page_nav",
                filepath = "main_for_on_link_click/page_nav.kt",
            )
        )
    }
    onOpenPage(pageId)
    return Result.success(Unit)
}
