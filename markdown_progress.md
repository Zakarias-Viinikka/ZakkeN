# markdown_progress.md

Written by AI for AI with zero memory. Companion to /home/zakke/ProgStuff/ZakkeN/markdown_continuation_plan.md (the task and its design). Keep this file current IN PLACE. If code and this file disagree, code wins. Created 2026-11-23.

The old files markdown_implementation_plan.md and markdown_implementation_progress.md were moved to old_stuff/. 19nov_progress.md is about the ANDROID task, ignore it. spike_for_markdown/SPIKE_REWRITE.md was NOT updated, Zakke decided notes like that are not relevant.

All paths below are relative to /home/zakke/ProgStuff/ZakkeN/cqrs_state_synchronization/web_interface/ unless absolute.

## Where we are

The CodeMirror markdown editor replaced the textarea in every block and works. The next task is in the plan file (custom [[command|rest]] clicks). Nothing of that task is written yet.

## What exists

- markdown_js/ : package.json plus src/{editor,decorations,link_parser,theme,wikilink_complete}.js. After changing any of these run `cd markdown_js && npm run build` (output markdown_web_output/cm_bridge.js, committed). No Trunk hook. Then `c c` / `c r`.
- index.html loads markdown_web_output/cm_bridge.js as a classic script.
- src/markdown_conversion/ : mod.rs, cm.rs (the ONLY file that talks to JS), edit.rs and keys.rs (pure logic, tests), markdown_conversion.rs (<MarkdownConversion />). Props mirror the old MyBlockContent. Key intents (arrows, Enter, Backspace, Delete) are decided in Rust in keys.rs.
- src/bbox_drag_reorder_container/my_block_content.rs : thin adapter around <MarkdownConversion /> (placeholder "Untitled" for title, "" otherwise, class "ps-content"). drag_reorder_container.rs is untouched.
- src/complex_state/caret/ : caret_request.rs (one request signal, CaretPlace = Start | End | At(utf16)), caret_handlers_registry.rs, focus.rs (at_end, at_start, at_pos). src/leptos_components/main_content/caret_owner.rs is <CaretOwner />, mounted once in main_content.rs. The converter registers a handler per block once its editor is ready.
- Styling: markdown_js/src/theme.js is a transparent dark theme that inherits font, size and color from .ps-content (so title size works). It sets .cm-scroller overflow visible (fixes a stray scrollbar on the title). In src/bbox_drag_reorder_container/drag_reorder_black_box/drag_reorder_black_box.css the focus rule is `.ps-content:focus-within`.

## Verified (Zakke said so or pasted output)

- c c builds fine.
- Looks right on the dark theme, title scrollbar gone.
- Typing, Enter split, Backspace merge, arrows between blocks, deleting an empty block, long wrapped lines, dragging: Zakke said "stuff seems to work fine" after trying the checklist.
- Heights: the drag-reorder box already has a ResizeObserver on every .ps-box, so editor growth is measured. Seen by reading code, behaviour confirmed only by Zakke's general "works".

## Not verified / known gaps

- Unit tests for keys.rs and edit.rs were never run.
- Holding Delete on an empty block logs "delete block failed ... found no block with id". Cause (my best guess, unconfirmed): key repeats reach the old editor before the caret moves to the block above, each asks to delete the same block again. Harmless. Possible fix: ask for the delete only once per block in markdown_conversion.rs. Zakke has not asked for it.
- Shift+Enter behavior was never compared to the old code.
- The wiki-link autocomplete popup was not styled for the dark theme. May look light.
- `titles` (wikilink title map) and the old on_link_click are unwired. The old click callback is replaced by the plan's (command, rest) callback.
- UTF-16 vs bytes vs chars mismatch (å ä ö, CJK, emoji) between the editor and yrs / text_diff is a real bug outside web_interface. Zakke said to ignore it for now. Details were in old_stuff/markdown_implementation_progress.md.

## Next steps

1. Start the plan's order of work, step 1. Before writing code, ask Zakke if anything in the plan is unclear. Read cm.rs and wikilink_complete.js first (not read this session).
