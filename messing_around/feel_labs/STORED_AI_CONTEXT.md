# feel_labs — AI stored context

Written by AI for AI. Meant to be the first file a fresh AI reads when
starting work on this project. Update a section when its content
changes and bump that section's date. If code and this file disagree,
the code wins and this file should be corrected.

## What this is — 2026-10-16

A Leptos CSR playground for deciding how typing and editing effects
should feel. Disposable experiment, not a product. No persistence, no
backend, no state carried between reloads. The user looks at it on
screen and says whether an effect feels right.

Under messing_around, which is UI style explorations and small Leptos
playgrounds, not part of the ZakkeN build.

## Layout — 2026-10-16

Stack: Leptos 0.8.22 with the csr feature, leptos_router 0.8.17,
stylist 0.15.1, wasm-bindgen, web-sys, console_error_panic_hook,
nightly toolchain, served with Trunk.

local_cargo.sh is the wrapper: `local_cargo.sh run` forwards to
trunk serve with the remaining args, anything else forwards to cargo.
So `local_cargo.sh check` is a cargo check.

src/main.rs is the picker and nothing else. It owns one selected-lab
signal and renders exactly one lab branch. Switching labs unmounts the
current component and mounts a fresh one, so returning to a lab starts
from its initial state. Do not change that to hide-and-show; timer
state has to reset.

src/ has one file per lab: block_transition, running_into_wall,
deleting_block, idle_page, new_page_popup, organic_split. Each owns its
signals, timers, listeners, and tunable constants. Nothing is shared
between labs.

styles/app.css holds the app baseline, the picker, and the styles for
the labs that need literal class names (see gotchas).

README.md describes each lab in plain terms.

## The labs — 2026-10-16

Block transition — a textarea plus an "Add block" button. The button
appends a line; both preview panes animate that one new line. "Soft
rise" lifts and fades; "Indent settle" slides in from the right with
overshoot. Only the new line animates.

Running into the wall — a pane that types text on a timer. When a word
causes a wrap to a new row, a bar slides in from the right, settles
under the row that just filled, drifts left, fades. One bar per wrap.
Row count is measured from client_height divided by a known line
height, after a short timeout so layout has settled.

Deleting a block — a list of blocks with delete buttons. Delete marks
the block red, holds, then fades and pops it. The block stays in the
signal-backed list through both phases and is removed only after the
exit timeout. Reset repopulates.

Idle page — after two seconds with no pointer movement, clicks, keys,
or scrolling, the page surface slides left 55px and reveals a stack of
sheets underneath, which sway. Any activity slides it back. Only the
background layer moves; content is a separate sibling above it.

New page popup — opens already showing a mock workspace with the dialog
over it. Live title preview above a focused input, Enter/Escape, click
on the overlay cancels. Each letter is its own keyed span so only newly
inserted letters shake.

Organic split — a rounded block that splits into two. Both rects
animate their corner radius to a full circle, the top lifts and the
bottom descends, an SVG metaball filter (blur + alpha color matrix)
thins and breaks the connection, and both settle back to squares.

## Gotchas — 2026-10-16

stylist gives one generated class name per style block, and that class
goes on the root element only. Child selectors written inside the block
do not match anything, because the children in the DOM do not carry the
hashed class. So a lab whose CSS uses `.foo .bar` silently gets no
styling. block_transition and deleting_block use stylist successfully
because their CSS is flat. The other four keep their CSS in app.css
with literal class names.

Style::new takes a Sheet, not a string. style! is the macro that takes
a string template and does ${name} substitution.

on_cleanup requires Send + Sync. A wasm_bindgen::Closure is neither, so
you cannot capture one in a cleanup. Use window_event_listener, which
returns a handle with a remove() method. set_interval_with_handle and
set_timeout_with_handle return the same shape.

The view! attribute parser chokes on a turbofish inside an attribute
value, for example each=move || (0..n).collect::<Vec<_>>(). Bind the
closure to a let first.

To restart a CSS animation, the element has to be remounted. Setting
the same boolean true repeatedly does nothing. The pattern used here is
a generation counter in a For key, so a new generation produces a new
node with a fresh animation.

The SVG filter region defaults to the element's bounding box, which
does not follow a CSS transform, so a shape that moves far enough gets
clipped during the animation and unclips when it stops. Fix with
filterUnits="userSpaceOnUse" and a fixed rect that covers the whole
travel.

leptos_router is a dependency but nothing imports it, so cargo warns
on every build. Left alone on purpose.

prefers-reduced-motion is honored in every lab, either by a media query
in CSS or by match_media on the window.
