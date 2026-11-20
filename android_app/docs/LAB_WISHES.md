# LAB_WISHES.md

Zakke's wishes for the Android app, collected 2026-11-20 in the interview-first workflow. Written for an AI with zero memory. Zakke does not read code or decide implementation details; he describes what he wants in plain words. If code and this file disagree, code wins.

## Verbatim answer (2026-11-20)

> i want the drag reorder system integrated. but without the handle to drag them manually. it's suppoed to just exist so the other things i want work properly to begin with. i want indent style block transition when u create a new block. or rather. it should activate when u type into an empty block instad of when u press enter.
>
> i want the drop effect when block deleted. i want the new page popup and the colors and whatnot from the labs into my app. also the new page popup needs to be fixed because the "Type a title" text doesnt get replace by what user types
>
> the main menu should replace the X button with the eye. and then it does the "drop animation" like 1.5 sec after the eye has been pressed.
>
> i want the popups.
>
> i want teh settings thng. and to move all the dev stuff to there. and then the way it's accessed is by a cog wheel in main menu. the dev stuff specifically for all the "annoying buttons that dont belong in main menu"

## Interpreted spec (AI's reading, correct it if wrong)

1. Drag-reorder engine: integrate it into the real app as an invisible foundation. No drag handle and no manual dragging by the user. It exists so the animations below (drop, indent transition) work.
2. Indent-style block transition: plays when the user types the first character into an empty block, not when Enter creates the block.
3. Drop effect when a block is deleted.
4. New page popup: port it and the lab's colours and look into the real app. Bug to fix: the "Type a title" placeholder does not disappear when the user types.
5. Main menu: replace the "X" delete button on each row with an eye. After the eye is pressed, the row plays the drop animation about 1.5 s later. (Assumed: the eye means hide or disable the page, and the drop animation is the row falling away. Confirm in code what the X does today, since it is probably delete or disable.)
6. Popups: bring the lab's popups into the real app.
7. Settings page: opened by a cog wheel in the main menu. All dev-only buttons ("annoying buttons that don't belong in the main menu") move there.

## Not answered (do not assume)

- Drag-across-blocks multi-select, what to do with selected blocks, undo/redo, scroll feel: not requested.
- Whether the lab should look wild or match the app: he wants the lab's colours and popups in the real app.

## Open questions for the next AI

- Which lab experiments hold the drag reorder, popups, new page popup and colours? Read `lab/` first.
- Does the eye hide, disable or delete? Check NavPage.kt.
