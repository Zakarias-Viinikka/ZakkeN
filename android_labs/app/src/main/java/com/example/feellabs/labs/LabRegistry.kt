package com.example.feellabs.labs

enum class LabGroup(val label: String) {
    Typing("Typing"),
    Blocks("Blocks"),
    PagesAndChrome("Pages and chrome"),
}

enum class Lab(
    val id: String,
    val label: String,
    val blurb: String,
    val group: LabGroup,
) {
    BlockTransition(
        id = "block_transition",
        label = "Block transition",
        blurb = "A new line is appended to two preview panes; each pane shows a different easing for how that line arrives.",
        group = LabGroup.Typing,
    ),
    RunningIntoWall(
        id = "running_into_wall",
        label = "Running into the wall",
        blurb = "Text types itself and three small dots trail across each row that fills.",
        group = LabGroup.Typing,
    ),
    SearchBar(
        id = "search_bar",
        label = "Search bar",
        blurb = "The look of typing into a search field with fake results appearing and updating below.",
        group = LabGroup.Typing,
    ),
    DeletingBlock(
        id = "deleting_block",
        label = "Deleting a block",
        blurb = "Delete a block. Three exit variants selectable via a segmented control; each locks per block at click time.",
        group = LabGroup.Blocks,
    ),
    BlockCascade(
        id = "block_cascade",
        label = "Block cascade",
        blurb = "Delete or insert a block and everything below moves. Three motion variants to compare.",
        group = LabGroup.Blocks,
    ),
    DragReorder(
        id = "drag_reorder",
        label = "Drag reorder",
        blurb = "Physics block reorder: drag between pages, split on Enter, merge on Backspace.",
        group = LabGroup.Blocks,
    ),
    NewPagePopup(
        id = "new_page_popup",
        label = "New page popup",
        blurb = "A mock workspace with a new-page dialog over it. Newly typed letters shake.",
        group = LabGroup.PagesAndChrome,
    ),
    MainMenu(
        id = "main_menu",
        label = "Main menu",
        blurb = "A page-list menu with a pen and eye per row. Three hover effects to compare.",
        group = LabGroup.PagesAndChrome,
    ),
    Popups(
        id = "popups",
        label = "Popups",
        blurb = "A toast stack. Click a popup to inspect it as a modal; auto-dismiss pauses while the modal is open.",
        group = LabGroup.PagesAndChrome,
    ),
    SettingsModal(
        id = "settings_modal",
        label = "Settings modal",
        blurb = "A designed settings modal. Six categories, a variety of control types, no save.",
        group = LabGroup.PagesAndChrome,
    ),
    ;

    companion object {
        fun grouped(): List<Pair<LabGroup, List<Lab>>> {
            return LabGroup.entries.map { g -> g to entries.filter { it.group == g } }
        }
        fun byId(id: String): Lab? = entries.firstOrNull { it.id == id }
    }
}
