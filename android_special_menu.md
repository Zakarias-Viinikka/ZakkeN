# Android Special Menu & Two-Marker Selection

## What the Demo Does
Demonstrates a multi-block text selection and action system working on top of Rapier2D physics-positioned cascade blocks:
- **Two Markers (1 & 2):** Can be placed at cursor positions in the same or different blocks. Order-independent (start/end determined by cascade document order).
- **Visual Highlight:** Spans across all text between the two markers (start block remainder, full intermediate blocks, end block prefix).
- **Double-Tap Trigger:** Double-tap on content hides keyboard and opens popup without moving the existing cursor.
- **Actions Menu:** Secondary popup page with Copy (to clipboard), Cut out, Delete, and Clear (clears markers).
- **Deletion Semantics:** Intermediate blocks between markers are deleted from state and physics controller; edge blocks remain separate.

---

## Lab Reference Implementation
Located in `/home/zakke/ProgStuff/ZakkeN/android_labs/special_menu/`:
- `app/src/main/java/z/special_menu/MarkerLogic.kt`
  - `TextMarker`, `SelectionAction`, `deriveSelection(...)`, `shiftMarkerOnEdit(...)`.
  - Pure Kotlin logic for deriving selection segments, copy text, and block deletion.
- `app/src/main/java/z/special_menu/MarkerVisualTransformation.kt`
  - `VisualTransformation` rendering marker badges ("1" blue, "2" green) and selection background highlight without modifying underlying raw strings.
- `app/src/main/java/z/special_menu/CenteredOverlay.kt`
  - Two-page popup: marker placement page ("1", "2") and selection actions page (Copy, Cut, Delete, Clear).
- `app/src/main/java/z/special_menu/MainActivity.kt`
  - State model (`LabState`), physics-positioned draggable blocks via `CascadeController`, double-tap interception, action dispatcher.

---

## Files to Touch in Main App (`android_app`)
- **Block Editor & Rendering:**
  - `app/src/main/java/z/zndroid/MainPages/ViewPage/EditableBlock.kt` (apply `MarkerVisualTransformation`, cursor/marker focus tracking).
- **Cascade & Physics Host:**
  - `app/src/main/java/z/zndroid/cascade/CascadeController.kt` & `CascadeColumn.kt` (block ordering, removals during Cut/Delete).
  - `app/src/main/java/z/zndroid/MainPages/ViewPage/ViewPage.kt` (attach double-tap gesture and overlay trigger).
- **Overlay & Actions:**
  - `app/src/main/java/z/zndroid/components/GlobalPopup.kt` or port `CenteredOverlay.kt` for the action dialog.
- **Lab Sandbox Reference:**
  - `app/src/main/java/z/zndroid/lab/experiments/selection/SelectionExperiment.kt`.
