# feel_labs - AI stored context

Written by AI for AI. Meant to be the first file a fresh AI reads when
starting work on this project. Update a section when its content
changes and bump that section's date. If code and this file disagree,
the code wins and this file should be corrected.

## What this is - 2026-10-28

A Leptos CSR playground for deciding how typing and editing effects
should feel. Disposable experiment, not a product. No persistence, no
backend, no state carried between reloads. The user looks at it on
screen and says whether an effect feels right.

Fourteen labs, picker grouped into Typing / Blocks / Pages and chrome.
Shell wraps each lab in a lab-head (title + blurb) and a `.lab-stage`
card.

Visual identity: dark slate + periwinkle, driven by CSS tokens in
`styles/app.css`. Dark only. `.mock-page` / `.mock-title` /
`.mock-block` are the shared primitives for anything that should feel
like a page.

Under messing_around, which is UI style explorations and small Leptos
playgrounds, not part of the ZakkeN build.

## Layout - 2026-10-28

Stack: Leptos 0.8.22 with the csr feature, leptos_router 0.8.17,
stylist 0.15.1, wasm-bindgen, web-sys, js-sys, gloo-timers, rapier2d,
console_error_panic_hook, nightly toolchain, served with Trunk.

local_cargo.sh is the wrapper: `local_cargo.sh run` forwards to
trunk serve with the remaining args, anything else forwards to cargo.
So `local_cargo.sh check` is a cargo check.

src/main.rs is the picker and shell. It owns one selected-lab signal
and renders exactly one lab branch inside a `.lab-stage` card, under
a `.lab-head` (title + blurb). Switching labs unmounts the current
component and mounts a fresh one, so returning to a lab starts from
its initial state. Do not change that to hide-and-show; timer state
has to reset.

src/ has one file per simple lab. `src/drag_reorder_lab/` is a
self-contained module directory, ported verbatim from
`cqrs_state_synchronization/web_interface`. It is the motion
reference: any new lab that moves blocks should imitate its cascade
constants or use the same easing curves.

styles/app.css holds the tokens, the shell, the shared primitives
(`.btn`, `.seg`, `.label`, `.toolbar`, `.mock-page`, `.mock-title`,
`.mock-block`), and per-lab styles that need literal class names.

README.md describes each lab in plain terms.

artifacts/ holds the dated briefs for past passes. The most recent
brief (BRIEF_2026-10-29_third_pass.md) sits at the root until the next
pass supersedes it, then moves into artifacts/. Briefs are history, not
active instructions; this file is the living one.

## The labs - 2026-10-28

Typing:
- Block transition - a textarea + "Add block"; the new line animates in
  both preview panes with two easings.
- Running into the wall - text types itself; a soft-ended line materialises
  and dissolves in place on each wrap (no travel).
- Search bar - mock search field with fake results; demo autoplays.
  Fake corpus is a const array (14 docs); no real search. Results body
  has a fixed height and scrolls; enter = rise (spring), exit = fall
  (ease-fall) then grid collapse.

Blocks:
- Deleting a block - three exit variants via segmented control
  (Red fade / Strike & slide / Drop). The variant is locked per-block
  at Delete-click time.
- Block cascade - delete/insert a block, everything below reflows.
  Segmented Fall / Spring / Ripple spring.
- Drag reorder - the physics block-reorder from the real app. Two mock
  pages side by side. Vacuum strip on the left stashes a block.
- Page entrance - a mock page whose title and blocks enter on mount.
  Segmented Rise / Drop / Unfold, plus Replay.

Pages and chrome:
- Idle page - three scenes for what a page does when left alone.
- New page popup - new-page dialog with shake-on-insert letters.
- Main menu - a page list with pen and eye per row; pen wiggles, eye
  closes and stays closed, three hover effects to compare.
- Popups - a toast stack. Click a card to open it as a scrollable
  modal; auto-dismiss pauses while a modal is open.
- Two-version page - pick one draft, edit it, confirm. Loser pops out,
  winner shines, confirmed page settles.
- Settings modal - six categories, a variety of control types, no save.
- Organic split - a block splitting into a new block, in the app's own
  style.

## Motion reference - 2026-10-28

`drag_reorder_lab/cascade.rs` is a gravity slide, not a spring:

- ACCEL 1500 px/s^2, MAX_V 600, DT 0.016.
- TRIGGER is GAP - CONTACT = -1, so the release gate (abs(gap) >=
  TRIGGER) is always true: all displaced blocks start together.
- The slide does not overshoot; it hard-stops at target and resets
  velocity on direction flip.

Block cascade exposes this exact feel as "Fall", plus "Spring"
(semi-implicit Euler, K 380, C 22, ~12% overshoot) and "Ripple spring"
(Spring with a 28ms per-rank stagger).

## Gotchas - 2026-10-28

stylist gives one generated class name per style block, and that class
goes on the root element only. Child selectors written inside the block
do not match anything, because the children in the DOM do not carry the
hashed class. So a lab whose CSS uses `.foo .bar` silently gets no
styling. block_transition and deleting_block use stylist successfully
because their CSS is flat. The other labs keep their CSS in app.css
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
node with a fresh animation. Used in main_menu (pen / eye icons),
search_bar (result rows), and page_entrance (the whole page).

The SVG filter region defaults to the element's bounding box, which
does not follow a CSS transform, so a shape that moves far enough gets
clipped during the animation and unclips when it stops. Fix with
filterUnits="userSpaceOnUse" and a fixed rect that covers the whole
travel.

Shared button rules in app.css (`.btn, .bt-btn, .rtw-btn, .npp-btn,
.os-btn, .db-btn`) use a group selector so every lab's buttons look
the same. Stylist rules have higher specificity than app.css; that is
why the local `.bt-btn` and `.db-btn` rules were deleted from the
stylist blocks in `block_transition.rs` and `deleting_block.rs`. Do not
reintroduce them.

No ancestor of a lab may create a stacking context or fixed-position
containing block (no transform, filter, backdrop-filter, contain,
will-change, opacity<1, isolation, z-index). new_page_popup's overlay
is position:fixed z-index:100 and must keep covering the whole viewport
above the sticky header. `.lab-stage` is a plain flex column for this
reason.

The mirror-textarea technique (for animating text in place while
keeping the caret stable) needs the mirror and the textarea to share
identical font, size, line-height, padding, border, letter-spacing,
white-space: pre-wrap, and overflow-wrap: break-word. Caret math from
the DOM is UTF-16; slice on a UTF-16 head and recompute the caret.
No lab uses this today — it was planned for an auto-correct lab that
was dropped before shipping.

SVG transforms need transform-box: fill-box (or view-box for groups
that want the whole SVG's coordinate system) plus transform-origin.
Without transform-box, `transform: rotate()` on an SVG path rotates
around the SVG canvas origin, not the path.

leptos_router is a dependency but nothing imports it, so cargo warns
on every build. Left alone on purpose.

prefers-reduced-motion is honored in every lab, either by a media query
in CSS or by match_media on the window.

## New labs this pass - 2026-10-29

- Popups (src/popups.rs): a global popup stack with per-popup
  expiry timestamps in `f64::INFINITY` while a modal is open (so
  nothing expires behind the modal), then re-armed to a fresh
  `now + LIFESPAN` on modal close. The reference under
  `leptos_components/popup` was the shape (stack + timed dismiss), not
  the feel. The modal is a fixed overlay, scrollable body, Escape /
  X / overlay close. Modal opens over the whole viewport, not just
  the stage.
- Two-version page (src/two_version.rs): four-phase machine
  (Choosing / Editing / Confirming / Confirmed). Variant is implicit in
  which version the user clicked, not a segmented control. Confirm
  triggers a `timeout` of `CONFIRM_TOTAL_MS` (1000ms) that flips to
  `Confirmed`. Pop-out uses --ease-spring (cubic-bezier(.34,1.56,.64,1))
  with rotate; shine is a `::before` pseudo-element with a skewed
  gradient sweep plus a scale-settle keyframe.
- Settings modal (src/settings_modal.rs): six categories rendered via
  a `match` on `Category`. All controls are backed by real signals so
  the modal feels interactive even though nothing saves. Danger zone
  uses --danger-soft. Save and Cancel both close (no persistence).
- The three new labs are *interaction* labs, not motion labs, but they
  use the same tokens and primitives as the rest.

## Motion additions - 2026-10-29

New curves added this pass (any future pass should reuse these instead
of inventing new ones unless a lab genuinely needs something else):

- `--ease-fall: cubic-bezier(0.11, 0, 0.5, 0)` (ease-in-quad,
  position ~ t^2; same shape as drag_reorder_lab/cascade.rs minus the
  velocity cap). Documented in the Motion tokens block in app.css.
  Derive duration from `t = sqrt(2d / 1500)` seconds for a distance d
  in px (54px -> ~268ms).
- `tv-pop-out-kf`: 0 -> 40% (scale 1.05, rotate -3deg) -> 100%
  (scale 0.2, rotate -15deg, opacity 0). The "pops out of existence"
  feel for the losing draft. Uses --ease-spring.
- `tv-shine-sweep`: skewed linear-gradient pseudo-element translating
  across the element, `--ease-out`. The shine on the confirmed draft.
- `sm-panel-in` / `pop-modal-in`: translateY(8px) + scale(0.98) ->
  rest, `--ease-out`. Standard modal-in. Use this for any new modal.
- `pop-card-in`: translateX(20px) -> 0, `--ease-out`. Standard toast-in.

## Motion additions (second pass) - 2026-10-28

- `--ease-fall: cubic-bezier(0.11, 0, 0.5, 0)` — ease-in-quad, gravity
  reference. Already in the Motion tokens block in app.css.
- `.btn-accent` — accent-tinted primary button. `.btn:disabled` — dim.
- `.sb-row` grid-collapse (1fr -> 0fr) wrapper for the search leave
  animation. Enter is a CSS animation (fill: backwards); exit is a
  transition toggled by `.leaving`. Leaving rows survive the commit
  until their own timer removes them, guarded by `leave_gen == this_gen`
  so later-leaving rows aren't cut off.
- `mm-eye-upper` folds via `scaleY(-1)` about `transform-origin: 50% 100%`
  (its own bottom edge), landing on the lower lid. State-driven from
  `row.disabled`, so the transition reverses cleanly.
- Organic split: shared keyframe percentages across all animations
  (pair-scale, half-translate, radius, neck-height) tied to
  `--os-total-dur`. No JS timers, only CSS.
- Idle page: no infinite animations on moving elements; state-driven
  transitions only (destination-state timing = slow idle entry, base
  rule timing = fast wake).

## Gotchas (second-pass additions) - 2026-10-28

- "gen" is a reserved keyword in edition 2024 — use `gen_id` or similar.
- `.sb-row-clip` / `.db-row-clip` must have `min-height: 0` and all
  vertical spacing must live INSIDE the clipped element (padding or
  margin on the item, not the clip), or a collapsed row leaves a gap.
- Fill-mode for enter animations must be `backwards`, never `both` /
  `forwards`, if the same element's exit state is class-driven. A held
  end value masks the exit.
- `Callback<T>` requires `T: Send + Sync`. `web_sys::DomRect` is not;
  unpack to primitives before crossing. Same for `wasm_bindgen::Closure`.
- Send-bound propagation: if a helper function takes `impl Fn() -> X`
  and that closure is captured by a `For` or `view!` closure, the
  parameter needs `+ Send + Sync + 'static`, or the whole tree fails.
- Pointer events fire on the element under the cursor even without
  movement; filter zero-delta pointermove before waking idle scenes.

## Motion additions (second pass) - 2026-10-28

- `--ease-fall: cubic-bezier(0.11, 0, 0.5, 0)` — ease-in-quad, gravity
  reference. Already in the Motion tokens block in app.css.
- `.btn-accent` — accent-tinted primary button. `.btn:disabled` — dim.
- `.sb-row` grid-collapse (1fr -> 0fr) wrapper for the search leave
  animation. Enter is a CSS animation (fill: backwards); exit is a
  transition toggled by `.leaving`. Leaving rows survive the commit
  until their own timer removes them, guarded by `leave_gen == this_gen`
  so later-leaving rows aren't cut off.
- `mm-eye-upper` folds via `scaleY(-1)` about `transform-origin: 50% 100%`
  (its own bottom edge), landing on the lower lid. State-driven from
  `row.disabled`, so the transition reverses cleanly.
- Running into the wall now uses three staggered dots (0/150/300ms
  delays) that translate -96px with ease-out and fade on arrival. Bars
  clean themselves up on `animationend`.
- Organic split: shared keyframe percentages across all animations
  (pair-scale, half-translate, radius, neck-height) tied to
  `--os-total-dur`. No JS timers, only CSS.
- Idle page: no infinite animations on moving elements; state-driven
  transitions only (destination-state timing = slow idle entry, base
  rule timing = fast wake).

## Gotchas (second-pass additions) - 2026-10-28

- "gen" is a reserved keyword in edition 2024 — use `gen_id` or similar.
- `.sb-row-clip` / `.db-row-clip` must have `min-height: 0` and all
  vertical spacing must live INSIDE the clipped element (padding or
  margin on the item, not the clip), or a collapsed row leaves a gap.
- Fill-mode for enter animations must be `backwards`, never `both` /
  `forwards`, if the same element's exit state is class-driven. A held
  end value masks the exit.
- `Callback<T>` requires `T: Send + Sync`. `web_sys::DomRect` is not;
  unpack to primitives before crossing. Same for `wasm_bindgen::Closure`.
- Send-bound propagation: if a helper function takes `impl Fn() -> X`
  and that closure is captured by a `For` or `view!` closure, the
  parameter needs `+ Send + Sync + 'static`, or the whole tree fails.
- Pointer events fire on the element under the cursor even without
  movement; filter zero-delta pointermove before waking idle scenes.
- Reactive attributes (e.g. `data-variant=move || ...`) must be closures,
  not snapshot values, if the underlying signal will change after
  mount. A static `let v = b.variant.attr(); ... data-variant=v` freezes
  the attribute forever; blocks won't restyle when their variant field
  updates. This bit us in deleting_block.rs and is fixed there now.

## Backlog - 2026-10-29

- Cursor spring (spring-follow pointer marker). Not built.
- Auto-correct lab. Was planned but dropped before shipping. The
  mirror-textarea technique is documented above in case the need comes
  back.
- Popups: no per-popup collapse animation on close (they just leave
  the Vec). If the stack ever gets crowded, an exit animation with a
  leaving-flag model (same pattern search_bar uses) is the right
  approach.
- Settings modal: no persistence, no live theme application (only
  local signal state). If this becomes a real settings screen, the
  modal shell is reusable.
- Two-version page: no undo of the pick, no per-block confirm; the
  whole page confirms at once. If finer-grained confirm feels right,
  that's a separate lab.