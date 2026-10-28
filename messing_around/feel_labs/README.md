# feel_labs

A small Leptos CSR playground for deciding how typing and editing effects
should feel. It is a disposable experiment, not a product: no persistence,
no backend, no state carried between reloads.

`./local_cargo.sh run` serves it with Trunk. Any other argument is forwarded
to cargo, so `./local_cargo.sh check` is a plain cargo check.

The picker groups the labs into Typing, Blocks, and Pages and chrome.
Switching away and back resets the lab - a lab that is not selected is
unmounted, not hidden. Each lab sits inside a shell with a title, a
one-line blurb, and a `.lab-stage` card.

`artifacts/` holds the dated briefs for each past pass
(`BRIEF_2026-10-28_visual_pass.md`, `BRIEF_2026-10-28_second_pass.md`;
see `BRIEF_2026-10-29_third_pass.md` at the root for the current one).
`STORED_AI_CONTEXT.md` is the living doc; briefs are history.

## The visual identity

Dark slate + periwinkle, driven by CSS tokens in `styles/app.css`. Mock
pages use `.mock-page` / `.mock-title` / `.mock-block` primitives.
`drag_reorder_lab`'s cascade is the reference for what satisfying block
motion feels like.

## Block transition

A textarea plus an "Add block" button. Clicking the button appends a new
line, and both preview panes animate that new line. "Soft rise" lifts it
from below and fades it in; "Indent settle" slides it in from the right
with a small overshoot. Only the newly added block animates; existing
lines are untouched.

## Running into the wall

A pane that types text on its own. When a word causes the text to wrap to
a new row, three small dots appear at the right edge of that row and
trail leftward, decelerating, fading out as they land. The dots are
staggered (0 / 150 / 300ms) so one is still in flight when the next
appears.

## Search bar

A mock search field with fake results. The "corpus" is a fixed set of
page-and-note hits; typing scores them by title, keyword, and snippet
match. A demo autoplays on load and stops when you type. Results are
highlighted, staggered in on mount, and hover-lit. Typing junk shows a
short "no pages match" line.

## Deleting a block

A short list of text blocks, each with a delete button. Three exit
variants selectable via a segmented control: "Red fade" (marks red,
holds, then fades and pops), "Strike & slide" (a strike line draws across
the text, then the row slides off to the right as siblings close the
gap), and "Drop" (the block lifts, then falls away with a slight tilt as
the row collapses). Each variant locks per-block at the moment Delete is
pressed, so changing the control mid-animation can't change an in-flight
row. A reset button repopulates the list.

## Block cascade

A mock page with editable text blocks. Deleting or inserting a block
reflows everything below with one of three motion variants: "Fall"
(mirrors the drag lab's gravity slide), "Spring" (bouncy overshoot),
"Ripple spring" (each displaced block waits 28ms per rank before starting).
Caret survives a delete above the focused block because blocks are keyed
and only their transform changes.

## Drag reorder

The physics block-reorder system from the real app, ported verbatim. Two
mock pages side by side; blocks can be dragged between them or ghosted
into the vacuum strip. Enter splits a block; Backspace at column 0 merges
into the one above. This lab is the reference for what satisfying block
motion feels like.

## Page entrance

A mock page whose title and blocks enter on mount. "Rise" (small overshoot,
55ms stagger), "Drop" (falls from above with a settle), "Unfold" (scaleY
from 0.6). Replay remounts the page; switching variant also replays. The
whole sequence is under a second.

## Idle page

Three scenes for what a page does when you leave it alone, selectable
via a segmented control. "Stack reveal" slides the top page left and
fans a stack of sheets out from behind it, each labelled and dimmed by
increasing depth. "Recede" pulls the sidebar and top bar away, shifts
the page to centre and scales it up, and darkens the edges. "Outline"
blurs the text away and grows skeleton bars in its place, approximating
each block's line shapes. Any activity (pointer, click, key, scroll)
wakes the page; entry is slow and staggered, return is prompt.

## New page popup

Opens already showing a mock workspace with a new-page dialog over it.
The dialog has a live title preview, a focused input, and Enter/Escape
handling. Newly typed letters shake as they are inserted; existing
letters do not. Enter confirms and leaves a result in the workspace,
Escape or a click on the overlay cancels. A "New page" button reopens it
after either outcome.

## Main menu

A page-list menu column, three times, each with a different hover effect:
"Underline draw" (accent underline grows in), "Nudge + pill" (title
nudges right, pill fades in), "Letter wave" (each letter lifts with a
stagger). The pen icon wiggles on click; the eye closes and stays closed
as a disabled signifier.

## Popups

A toast stack anchored to the bottom-right corner of the viewport. Toasts
arrive on their own (with a jittered cadence) and can be spawned by hand.
Hover any toast to pause the stack; click any toast to open it as a modal
in the middle of the screen — the modal body scrolls if the message is
long. While a modal is open, auto-dismiss is paused and nothing behind it
disappears. Close via X, Escape, or the overlay. Each toast carries a
depleting timer bar in its kind colour (info / success / warning / error).

## Two-version page

Two drafts of the same page side by side. Click one to pick it; the other
dims. The picked one becomes editable, block by block, with an
autosizing mirror-textarea so the page height stays correct as you type.
Change pick puts both cards back in the pool (edits preserved). Confirm
shatters the draft into 24 tiles that fly outward while the settled page
fades in underneath, a ring shockwave follows the border, and a shine
sweeps across at 620ms. The confirmed page settles into a green border.

## Settings modal

A designed settings modal. Six categories in a sidebar (Appearance,
Notifications, Editor, Account, Keyboard shortcuts, Advanced), a sliding
active indicator, and a dirty dot on any category with unsaved changes.
Every control is real (toggles, sliders, radios, dropdowns, swatches,
text inputs) and drives a live preview inside the Appearance pane. Save
morphs the backdrop page via scoped `--st-*` custom properties; Cancel
discards; closing with unsaved changes is refused and pulses the chip.

## Organic split

A block splitting into a second block. On trigger, the seam lights up,
inner corners round as the two halves pull apart, a neck stretches
between them and pinches, then snaps. The top half recoils and the two
settle 14px apart with all four borders lit. Split replays from the
joined state; Reset returns to joined.