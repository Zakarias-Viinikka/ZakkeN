# Zndroid UX Philosophy & Intent

This document serves as a "Ground Truth" for how the user interface and experience should feel and behave.

## 📱 Mobile-First Professional Editing
The goal is to create a pro-grade block-based editor (similar to Notion or Obsidian) that is actually pleasant to use on a phone.

### 1. Block-Based Interaction
- **Individual Blocks**: Every paragraph, heading, or item is its own block. This allows for structural manipulation (moving, splitting, merging) rather than just character editing.
- **Long-Press is Key**: On mobile, we avoid cluttered toolbars. Holding down on a block should open a **Block Action Menu**.
  - The menu should show the block in focus.
  - Quick action buttons (H1, Todo, Delete, etc.) should be positioned conveniently for thumb access.

### 2. The "Subtle Slash" (/)
- While traditional editors use `/` to open a big search menu, Zndroid uses it as a quick trigger.
- Typing `/` at the start of a block shows a **subtle popup**.
- Clicking this popup opens the full **Block Action Menu**.
- The popup disappears if the user continues typing something else or clicks away.

### 3. Multi-Block Selection & Navigation
- **Trail Selection**: When in selection mode, dragging a finger across blocks leaves a visual "trail" (currently a cyan/magenta fading line). This is for selecting multiple blocks for batch operations.
- **Edge Scrolling**: Dragging near the top or bottom 15% of the screen while in selection mode triggers auto-scrolling. This allows for multi-selecting across long documents without lifting the finger.

### 4. Fluidity & Flow
- **Focus Management**: Creating a new block (Enter) should automatically focus that new block. Merging blocks (Backspace at pos 0) should focus the junction point. The user should never have to manually tap a text field to keep typing during structural changes.
- **Ghost Text**: Empty blocks should provide a gentle hint ("Type '/' for commands...") to guide the user without being intrusive.

### 🧪 The Lab Protocol
- Crazy UX ideas are always built in the **AI Lab** folder first.
- We iterate on the "feel" (animations, finger tracking, timers) in isolation before moving any logic into production components.
- Governed by `AI_LAB_RULE.md`.

## 🛠️ Technical Underpinnings for UX
- **No-Lag Persistence**: We use a **Buffered & Squashed** approach. Character edits are collected and merged in the background every 500ms. The UI remains 100% responsive because it doesn't wait for SQLite or Yrs logic on every single keystroke.
- **CRDT Stability**: Because we use Yrs, undos/redos and multi-device sync are robust, but the UX should hide this complexity.
