# Zndroid UX Philosophy & Intent

This document serves as a "Ground Truth" for how the user interface and experience should feel and behave.

## 📱 Mobile-First Professional Editing
The goal is to create a pro-grade block-based editor (similar to Notion or Obsidian) that is actually pleasant to use on a phone.

### 1. Block-Based Interaction
- **Individual Blocks**: Every paragraph, heading, or item is its own block. This allows for structural manipulation (moving, splitting, merging) rather than just character editing.
- **Long-Press is Key**: On mobile, we avoid cluttered toolbars. Tapping the small `::` handle on the left of a block opens a **Block Action Menu**. The menu shows the block in focus and offers quick actions (H1, Todo, Delete, Move Up, Move Down) positioned for thumb access. Title blocks have the move options disabled (a title never moves).
- **Move up / down instead of drag**: On a phone, drag-to-reorder competes with text selection and scroll. Up / Down in the action menu is a two-tap reorder and doesn't need a drag target.

### 2. The "Subtle Slash" (/)
- While traditional editors use `/` to open a big search menu, Zndroid uses it as a quick trigger.
- Typing `/` at the start of a block shows a **subtle popup**.
- Clicking this popup opens the full **Block Action Menu**.
- The popup disappears if the user continues typing something else or clicks away.

### 3. Multi-Block Selection & Navigation
- **Trail Selection**: When in selection mode, dragging a finger across blocks leaves a visual "trail" (currently a cyan/magenta fading line). This is for selecting multiple blocks for batch operations.
- **Edge Scrolling**: Dragging near the top or bottom 15% of the screen while in selection mode triggers auto-scrolling. This allows for multi-selecting across long documents without lifting the finger.
- **Arrow keys jump blocks**: On a hardware keyboard, Down at the very end of a block focuses the start of the block below, Up at position 0 focuses the end of the block above. Modifier keys (Shift/Ctrl/Alt/Cmd) skip this so selection keeps working. Mostly a hardware-keyboard feature; the arrows do nothing extra on the on-screen keyboard.

### 4. Fluidity & Flow
- **Enter splits at the cursor**. Works on both hardware Enter and the on-screen newline. Text before the cursor stays in this block; text after moves to a new block whose focus is at position 0. Works in the title too, so a title can be split off into a body block.
- **Backspace at position 0 merges**. Backspace in a non-empty body block merges it into the block above and puts the cursor at the join point. Backspace in an empty body block deletes it and focuses the end of the block above. Backspace at position 0 of the title, or of the first body block (which sits below the title), does nothing.
- **Focus Management**: Structural actions set the focus so the user never has to tap to keep typing. Enter focuses the new block at position 0; merge focuses the join point; delete focuses the nearest surviving block.
- **Ghost Text**: Empty blocks provide a gentle hint ("Type '/' for commands...") to guide the user without being intrusive. Empty titles read "Untitled" in the nav menu (muted colour).
- **Flush on leave**: leaving a page always saves. Top-bar back, system back, lifecycle ON_STOP, and ViewPage's onDispose all flush before releasing the page. If a flush fails, the page reloads from DB and a popup warns the user. On a phone we can't tell the user to "refresh the page" the way web can; the reload is automatic.

### 🧪 The Lab Protocol
- Crazy UX ideas are always built in the **AI Lab** folder first.
- We iterate on the "feel" (animations, finger tracking, timers) in isolation before moving any logic into production components.
- Governed by `AI_LAB_RULE.md`.

## 🛠️ Technical Underpinnings for UX
- **No-Lag Persistence**: Character edits go through an ordered queue with a 500ms debounce. Consecutive edits to the same block are squashed before the transaction. Structural actions (split, merge, delete, move) go through the same queue, so they can't run ahead of pending text. The UI stays responsive because nothing waits on SQLite or yrs per keystroke.
- **CRDT Stability**: Because we use Yrs, undo/redo and multi-device sync are robust, but the UX should hide that complexity.
- **Page disable is a hide, not a delete.** The nav row's "Delete" button removes the page from the menu without touching any data. Blocks, blob and outbox rows stay. The button label is frozen for the e2e harness and does not reflect the new meaning.