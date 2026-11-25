package z.zndroid.MainPages.special_menu

/** A page whose title matched a query. Mirror of web's TitleMatch. */
data class TitleMatch(val title: String, val pageId: String)

/** Options the keyword picker can surface. MakeNewPageAndLinkToIt is a placeholder (web's NewPage is a stub too). */
sealed class MenuOption {
    object MakeNewPageAndLinkToIt : MenuOption()
    object LinkToExistingPage : MenuOption()
}
