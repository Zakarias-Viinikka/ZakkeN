# feel_labs

A small Leptos CSR playground for deciding how typing and editing effects
should feel. It is a disposable experiment, not a product: no persistence,
no backend, no state carried between reloads.

`./local_cargo.sh run` serves it with Trunk. Any other argument is forwarded
to cargo, so `./local_cargo.sh check` is a plain cargo check.

The picker at the top mounts one lab at a time. Switching away and back
resets the lab — a lab that is not selected is unmounted, not hidden.

## Block transition

A textarea plus an "Add block" button. Clicking the button appends a new
line, and both preview panes animate that new line. "Soft rise" lifts it
from below and fades it in; "Indent settle" slides it in from the right
with a small overshoot. Only the newly added block animates; existing
lines are untouched.

## Running into the wall

A pane that types text on its own. When a word causes the text to wrap to
a new row, a short bar slides in from the right, settles under the row
that just became full, then drifts left and fades out. One bar per wrap.

## Deleting a block

A short list of text blocks, each with a delete button. Deleting marks
that block red first, holds briefly, then fades and pops it away. The
block stays in the list through both phases and is removed only after the
exit finishes. A reset button repopulates the list.

## Idle page

After a couple of seconds with no pointer movement, clicks, keys, or
scrolling, the page surface slides left and reveals a stack of sheets
underneath it, which sway gently. Any activity slides the surface back.
Only the background layer moves; the sample content stays crisp and
stationary.

## New page popup

Opens already showing a mock workspace with a new-page dialog over it.
The dialog has a live title preview, a focused input, and Enter/Escape
handling. Newly typed letters shake as they are inserted; existing
letters do not. Enter confirms and leaves a result in the workspace,
Escape or a click on the overlay cancels. A "New page" button reopens it
after either outcome.

## Organic split

A rounded block that splits into two. On trigger, both shapes round into
bubbles, the top one lifts and the bottom one descends, and an SVG
metaball filter makes the connection between them thin and break. Both
shapes settle back to squares at the end. Replay restarts the whole
thing.
