# feel_labs — visual + labs pass, 2026-10-28

Durable record of the brief this pass was built from. Kept so a future
AI (or a future you) can see what was decided, in what order, and what
was out of scope. If the code and this file disagree, the code wins;
update this file when the shape changes.

The brief below is the exact instruction handed to the AI that built
this pass, pasted verbatim. Implementation decisions beyond it, and
anything cut, live in that AI's final report (not stored here). The
project's own living context lives in STORED_AI_CONTEXT.md — this file
is a point-in-time spec, not a replacement for it.

---

You are implementing a visual and content pass on feel_labs (Leptos 0.8 CSR playground, repo root is feel_labs/). Read STORED_AI_CONTEXT.md first, then src/main.rs, styles/app.css, and src/drag_reorder_lab/cascade.rs. Everything below is already decided. Do not ask the user questions; make the call and mention it in your final report. Run `./local_cargo.sh check` after each numbered step. The unused leptos_router warning is intentional.

HARD RULES (from the project)
- Do not touch cqrs_state_synchronization/web_interface. Do not port more cqrs code. Do not change the picker's unmount-on-switch behaviour. No persistence, backend, routing or carried state.
- Existing labs: styling only. No behaviour changes, no motion/timing changes, no markup changes beyond what is explicitly allowed below.
- stylist classes only land on the root element, so child selectors must live in styles/app.css with literal class names. Bind closures to a let before using turbofish in view! attributes. Restart a CSS animation by remounting through a generation counter in a For key. SVG filters use userSpaceOnUse with a fixed rect. on_cleanup needs Send+Sync, so use window_event_listener handles and TimeoutHandle/IntervalHandle .clear(). Honor prefers-reduced-motion in every lab (CSS media block in app.css, or match_media in Rust; copy the small prefers_reduced_motion helper from deleting_block.rs).
- If a web-sys method fails to compile for a missing feature, add the feature to Cargo.toml. That is allowed.

DECISIONS ALREADY MADE
- Visual direction: one cohesive identity derived from the user's real app. It is the dark slate + periwinkle look drag_reorder_black_box.css already has. Apply it via tokens. Dark only (drop the light scheme). Structure and DOM of existing labs stay.
- New labs (12 total after this): Main menu, Search bar, Block cascade, Auto-correct, Page entrance. Cursor spring is NOT built; list it as a backlog idea in STORED_AI_CONTEXT.md.
- Build order: 1 shell+tokens, 2 restyle existing, 3 Main menu, 4 Search bar, 5 Block cascade, 6 Auto-correct, 7 Page entrance, 8 docs. If you run long, cut from the end (Page entrance first) but always do the docs.

CORRECTION TO THE PRE-PLAN
The drag lab's cascade is NOT springy and does not overshoot. cascade.rs is: constant acceleration (ACCEL 1500 px/s^2), velocity capped (MAX_V 600), fixed DT 0.016 on a 16ms interval, hard stop at target, velocity reset if direction flips. TRIGGER is GAP-CONTACT = -1, so the release gate (abs(gap) >= TRIGGER) is always true and all displaced blocks start together: a lockstep gravity slide ("falling up"). Do not try to "fix" it (out of scope). The Block cascade lab below exposes that exact feel plus two springier alternatives so the user can compare.

=== STEP 1: SHELL AND TOKENS (styles/app.css top, src/main.rs) ===

Tokens on :root (color-scheme: dark):
- Surfaces: --bg #0f141d, --surface-1 #151c29 (shell cards/stage), --surface-2 #1c2433 (mock pages, matches .ps-page-panel), --surface-3 #2b3648 (raised/hovered, matches dragging box), --bg-deep #0b1020 (organic split stage).
- Text: --fg #e4e9f2, --fg-dim rgba(228,233,242,.72), --fg-faint rgba(228,233,242,.42).
- Accent: --accent #8ab4ff, --accent-soft rgba(138,180,255,.14), --accent-line rgba(138,180,255,.5), --focus-ring rgba(138,180,255,.45).
- Lines: --border rgba(228,233,242,.09), --border-strong rgba(228,233,242,.2).
- Status: --danger #ff6b6b, --danger-soft rgba(255,107,107,.14).
- Radii --r-sm 6px, --r-md 10px, --r-lg 14px. Shadows --shadow-raise 0 24px 48px -18px rgba(0,0,0,.75), --shadow-soft 0 8px 24px -12px rgba(0,0,0,.6).
- Type: --font-ui (existing system-ui stack, no web fonts, no network), --font-mono (ui-monospace stack). Scale 11/13/15/18/26px. Big titles weight 700, letter-spacing -0.03em. Small "label" style: 11px, uppercase, 0.12em tracking, --fg-faint.
- Motion: --ease-out cubic-bezier(.22,1,.36,1), --ease-spring cubic-bezier(.34,1.56,.64,1), --dur-fast 120ms, --dur-med 220ms.
- Keep the OLD variable names (--page-bg, --page-fg, --panel-bg, --picker-active-*, etc.) as aliases to the new ones so nothing breaks mid-migration.

Shell (main.rs + CSS):
- Sticky header (z-index 10): small wordmark "feel_labs" with an accent dot, then the picker as pill buttons grouped in three clusters with tiny group labels: Typing = Block transition, Running into the wall, Auto-correct, Search bar. Blocks = Deleting a block, Block cascade, Drag reorder, Page entrance. Pages and chrome = Idle page, New page popup, Main menu, Organic split. Add `Lab::group()` and reorder LABS accordingly. Default selected lab stays Block transition.
- main.lab-area: max-width ~1180px, centered, padding ~28px 24px 64px. Inside: a lab head (lab name as 26px title + one-sentence blurb in --fg-dim from a new `Lab::blurb()`, adapt from README text), then a `.lab-stage` card (surface-1, border, --r-lg) wrapping the existing reactive `match`. The match stays inside the reactive closure, exactly as today, so unmount-on-switch is preserved.
- `.lab-stage` must be a flex column so existing `flex:1` labs still fill it. overflow:hidden is fine.
- CRITICAL: no ancestor of a lab may create a stacking context or fixed-position containing block (no transform, filter, backdrop-filter, contain, will-change, opacity<1, isolation, z-index). new_page_popup's overlay is position:fixed z-index:100 and must keep covering the whole viewport above the sticky header.

Shared primitives in app.css (new labs use them; existing labs get them via group selectors):
- One button style applied via a group selector to `.btn, .bt-btn, .rtw-btn, .npp-btn, .os-btn, .db-btn`: ghost button, --border, --r-sm, hover --border-strong + --surface-3 background, focus-visible ring. Then DELETE the local .bt-btn/.db-btn rules from the stylist blocks in block_transition.rs/deleting_block.rs. Stylist rules have higher specificity than app.css and would otherwise win.
- `.seg` / `.seg-btn` / `.seg-btn.active`: segmented control (accent-soft active).
- `.toolbar` (flex, wrap, gap), `.label` (the small-caps style).
- `.mock-page` (surface-2, 1px --border, top border --accent-line, --r-lg, padding), `.mock-title` (clamp 1.8-2.6rem, 700, tracking -0.04em), `.mock-block` (transparent, 1px border rgba(228,233,242,.04) brightening to .09 on hover, --r-sm, padding like the drag lab's .ps-box, color --fg-dim, line-height 1.6).

=== STEP 2: RESTYLE EXISTING LABS (CSS/inline-color edits only) ===
- Block transition (stylist block): replace rgba(128,128,128,...) with tokens; panes are surface-2 cards; pane titles use the .label style; textarea surface-1 with mono font. Keep every keyframe and duration.
- Running into the wall: pane = surface-2; bar gradient uses the accent; "rows: n" faint with tabular-nums.
- Deleting a block (stylist block): blocks as surface-2 rows with --border; .db-red uses --danger (and --danger-soft background); exit animation unchanged.
- Idle page: the sheets become tonal steps of the slate palette (edit the tone strings in UNDER_SHEETS and .ip-sheet-top), each with a 1px --border so the stack stays legible against dark. .ip-content text becomes --fg (it is hardcoded dark-on-white today, so it MUST change with the sheets). Layout, offsets, delays untouched.
- New page popup: style .npp-root/.npp-workspace-* with the .mock-page look using CSS only (no markup changes). Panel/border/radius use tokens. Overlay stays fixed.
- Organic split: .os-stage background var(--bg-deep), .os-svg color var(--accent). Filter and animation untouched.
- Drag reorder: leave the lab alone. Optionally swap its exact hardcoded hexes for the matching tokens in drag_reorder_black_box.css (values are equal by design, so no visible change). Do not touch any .rs there.

=== STEP 3: MAIN MENU (src/main_menu.rs, prefix mm-) ===
Purpose: feel of a page-list menu. Three identical menu columns side by side (stack on narrow screens), each labelled with its hover-effect name, so the user compares hover effects directly. Each column is a surface-2 card headed "Pages" with 6 rows. Page titles must be realistic: Weekly planning, Reading list, Trip itinerary, Recipes to try, Meeting notes, Budget 2026.
- Row: title on the left, pen button and eye button on the right (inline hand-written 24x24 stroke SVGs: stroke=currentColor, width ~1.75, round caps; pen = pencil outline, eye = almond outline + pupil circle). Icons sit at ~0.5 opacity and go to 1 on row hover or focus-visible. They are real <button>s with aria-labels.
- Pen click: the pen wiggles (rotate around the writing tip, damped: 0, -14deg, 11deg, -7deg, 4deg, 0, ~520ms, --ease-out). Eye click: the eye blinks (scaleY of the eye group to ~0.1 and back, ~260ms ease-in-out). SVG transforms need transform-box: fill-box with transform-origin set. The eye click also toggles that row "disabled": the row dims to ~0.45 and a diagonal slash on the eye draws in (stroke-dashoffset transition), so the blink reads as the state change. No other functionality (no rename).
- Restart animations by remounting the icon: each icon wrapper lives in a For over a one-element vec holding that icon's generation counter, keyed on it. Apply the animation class only when gen>0, so nothing wiggles on mount. Keep the counters per row in a small per-row signal struct.
- Hover effects, selected by a static data-hover attribute on each column, CSS in app.css keyed on that attribute. Hover is triggered by `:hover` on the row (not focus-within, which would stick after a click):
  1. "Underline draw": accent underline grows left to right (linear-gradient background-size transition), text shifts toward the accent.
  2. "Nudge + pill": title translates ~4px right with --ease-spring, and an accent-soft pill fades in behind the row.
  3. "Letter wave": the title is rendered as per-character spans with an --i index var (use white-space: pre so spaces survive); on hover each letter lifts ~2px and tints accent with transition-delay calc(var(--i) * 18ms). It staggers back on mouse-out because it is a transition.
- Tunables as consts at the top (WIGGLE_DEG, WIGGLE_MS, BLINK_MS, LETTER_STAGGER_MS), injected as CSS vars via inline style on the root (the new_page_popup pattern).

=== STEP 4: SEARCH BAR (src/search_bar.rs, prefix sb-) ===
Purpose: the LOOK of typing into a search field with fake results appearing and updating. No real search. The "corpus" is a const array of ~12 canned entries styled like the user's own pages and notes: each has id, page title, a breadcrumb ("Page > block" style), a 1-2 sentence snippet, and keywords. Snippets must read like real hits with concrete detail, not lorem ipsum (example: title "Budget 2026", snippet "Move the streaming and gym subscriptions to annual billing, saves about 14% over twelve months."). Cover topics like trip planning, budget, meeting notes, reading list, recipe, workout log, project retro, gift ideas, apartment search, book notes, password rotation checklist, weekly review. Do not mention any real-world location of the user.
- Layout: a surface-2 mock page containing: input row (magnifier svg, a real <input>, a "/" kbd chip that hides when text is present, a clear button that appears when non-empty) with accent focus-glow transition; a thin shimmer line under the input that sweeps while "searching"; a results card below with an "N results" label (tabular-nums) and up to 4 result rows (title, breadcrumb, snippet with matched words highlighted via <mark>-style spans), the first row styled as the active/selected one, all rows with :hover; a static footer hint. Give the results card a min-height of 4 rows so nothing jumps.
- Pipeline: `query` signal feeds the input (Effect syncing input value, the block_transition textarea pattern). On every change: searching=true, clear the previous debounce handle, set_timeout_with_handle ~140ms, then compute results and set a `committed_query` + `results` together and searching=false. Scoring: lowercase tokens; title hit +3, keyword hit +2, snippet hit +1; keep score>0, sort desc, top 4. Empty query shows "Recent" (first 4). Non-empty with no matches shows a short "No pages match, try 'trip' or 'budget'" line. Highlight against committed_query so highlight and results update together. Use to_ascii_lowercase for matching so byte offsets stay valid, then slice on those offsets.
- Results list is a keyed For (key = hit id). Newly mounted rows rise+fade in with a small stagger (--i set at mount, ~30ms per index). Rows that stay do not re-animate (snippet highlight just updates). Leaving rows may disappear instantly.
- Autoplay demo runs on mount: ONE interval (~85ms) driving a tiny state machine over a list of queries (e.g. "trip", "budget october", "reading list"): types char by char with a little timing jitter, holds ~1.4s, erases quickly, next query, loops. It writes the same `query` signal. Programmatic set_value fires no input event, so the input's on:input only fires for real user typing: stop the demo there and on focus. A "Replay demo" button restarts it. Clear the interval in on_cleanup.

=== STEP 5: BLOCK CASCADE (src/block_cascade.rs, prefix cl-) ===
Purpose: show the drag lab's block-motion feel outside the drag lab: delete or insert a block and everything below moves. A segmented control picks the motion variant live (switching does not reset the page):
- "Fall": exactly the drag lab's integrator. ACCEL 1500, MAX_V 600, DT 0.016, vel reset to 0 on direction flip, hard stop at target. Comment that it mirrors drag_reorder_lab/cascade.rs.
- "Spring": semi-implicit Euler toward target: a = -K*(y-t) - C*v, K~380, C~22 (about 12% overshoot), settle-snap when |y-t|<0.2 and |v|<5. Clamp |v| to a sane max.
- "Ripple spring": Spring, but each displaced block waits rank*STAGGER_MS (28ms, rank = index among displaced blocks counting down from the first one next to the change) before starting, via a per-block delay countdown in the same tick.
Structure: a 560px-wide `.mock-page` with a fixed title row (not deletable) and ~6 text blocks of mixed heights. Use realistic note-like sentences, not lorem. Heights are explicit per block (lines*24+20, text pool sized to fit 1-3 lines at that width) so NO DOM measuring is needed. Blocks are absolutely positioned inside the page with transform: translateY(y). Block text is an editable <textarea> (uncontrolled, prop:value=initial, fixed height, overflow hidden, resize none). On hover each block shows an x (delete) and a + (insert a new block below, text from a pool, height cycling through the three sizes). Reset button restores the initial page by snapping. Refuse a delete if it would leave zero non-title blocks.
Mechanics: Vec<Body> in one RwSignal with id, text, h, y, target, vel, delay, phase (Idle/Moving), dying. A pure `layout` pass computes targets top-down (TOP_PAD + running h + GAP) over non-dying bodies; any body whose y != target becomes Moving (with delay in Ripple). Insert: new body starts at its own target (CSS pop-in on mount); bodies below fall down. Delete: mark dying (CSS 140ms fade/scale-out, excluded from layout), others retarget immediately, remove after a tracked timeout. A 16ms set_interval_with_handle runs the step and only calls `set` when something moved (same pattern as the drag lab's black box). Page height = max(y,target)+h+padding over bodies, like the drag lab's stage_height, so it sags in sync. Render with a keyed For (key = id) and per-body closures that read y by id; never remount a block when it moves. Reduced motion: snap y=target on every layout.
Caret requirement: because blocks are keyed and only transform changes, a focused textarea keeps focus and caret while blocks above it are deleted. Verify this manually (click into a block, delete the block above, keep typing).

=== STEP 6: AUTO-CORRECT (src/auto_correct.rs, prefix ac-) ===
Purpose: typos get smoothed into the corrected word, in place, with a spring pulse, and the caret never jumps. Segmented control with two variants: "Pulse" (corrected word scales 1 to ~1.22 to ~0.96 to 1 with --ease-spring while tinting accent back to --fg) and "Swap" (the old typo drifts up and fades out as the corrected word rises in from below).
Technique: mirror textarea. A grid container with one cell: a mirror div and a real textarea stacked in the same cell. They must have IDENTICAL font, size, line-height, padding, border, letter-spacing, white-space: pre-wrap, overflow-wrap: break-word. The textarea is absolutely filled, color: transparent, caret-color var(--fg), transparent background, resize none, overflow hidden, with a translucent ::selection; its ::placeholder stays faint and visible. The mirror is pointer-events:none, shows the same text, and appends a trailing newline sentinel so a final empty line has height; the mirror therefore drives the height. Style it like a .mock-block inside a .mock-page.
Mirror rendering: split text into word tokens (runs of non-whitespace) and whitespace tokens. Render with a keyed For, key = (token index, pulse generation if that token is the pulsed one else 0); each token reads its text reactively by index (the block_transition by-index pattern), so typing never remounts unchanged tokens. Only the corrected token gets a `.ac-fix` span (inline-block while animating, position:relative, data-old attribute for the old text). The ghost old word is a ::before with content: attr(data-old). A ~700ms timeout clears the pulse afterwards (track the handle, clear in on_cleanup). Accept that editing earlier text inside that window could mis-pulse a neighbouring token.
Correction logic on each input event: keep prev_value in a StoredValue. Trigger only when the new value is exactly prev plus ONE inserted delimiter char (space, newline, or . , ! ? ; :) immediately before the caret, so deleting, pasting and backspacing never retrigger. Take the maximal letters/apostrophes run just before that delimiter, lowercase it, look it up in a const typo table (teh>the, adn>and, taht>that, recieve>receive, seperate>separate, definately>definitely, wierd>weird, thier>their, becuase>because, freind>friend, occured>occurred, untill>until, tommorow>tomorrow, accomodate>accommodate), preserve a leading capital. Selection offsets from the DOM are UTF-16 units: do the slicing on a UTF-16 head (value up to the caret) and set the new caret to old_caret + (utf16_len(fix) - utf16_len(typo)), since the edit lies wholly before the caret. After textarea.set_value(new) the browser moves the caret to the end, so call set_selection_range immediately (the same idea as map_caret_across_edit in drag_reorder_lab/my_block_content.rs; hand-copy the logic, do not import). Programmatic set_value clears native undo; that is acceptable here.
Test cases: "teh " mid-sentence keeps the caret right after the typed space; "accomodate " (+1 length) keeps the caret correct; capital "Teh " gives "The". Show a one-line hint: type a sentence with "teh", "adn" or "recieve".

=== STEP 7: PAGE ENTRANCE (src/page_entrance.rs, prefix pe-) ===
Purpose: a mock page whose title and blocks enter on mount. Segmented control: "Rise" (each block rises ~18px with a small overshoot, stagger ~55ms), "Drop" (falls in from above using an ease-in-then-settle curve echoing the gravity feel, title first), "Unfold" (scaleY from 0.6, transform-origin top, fade, stagger). Replay button, and switching variant also replays. Replay by remounting the page through a generation counter in a For key. CSS keyframes in app.css with --i index vars. Page = .mock-page with a title and 6 realistic blocks. Entire sequence under ~900ms. Reduced motion becomes a plain fade.

=== STEP 8: DOCS ===
- README.md: lab count and a short plain-language entry for each new lab; mention the Drag reorder lab (it is missing today) and the shell/identity.
- STORED_AI_CONTEXT.md: add or bump sections dated 2026-10-28 for: What this is (12 labs; the visual identity and where tokens live), Layout (new files, drag_reorder_lab/ is a self-contained module dir that is the motion reference, grouped picker, lab-stage), The labs (the 5 new ones plus the drag lab), and Gotchas. Add these gotchas: (a) cascade.rs is a gravity slide with a hard stop and TRIGGER is negative (always-open gate), so it does not overshoot or ripple; (b) shared button rules must not fight stylist-block rules on specificity; (c) the stage must not create a stacking context or containing block or the NPP fixed overlay breaks; (d) the mirror-textarea technique and UTF-16 caret math; (e) SVG transform needs transform-box: fill-box. Add a backlog line: Cursor spring (spring-follow pointer marker) not built.

=== DONE CHECKLIST ===
`./local_cargo.sh check` is clean (only the leptos_router warning). Then eyeball in the browser:
- Every old lab behaves exactly as before. NPP overlay covers the viewport including the header. Idle page text is readable.
- Main menu: pen wiggles and eye blinks on every click (repeat clicks restart the animation); the three hover effects are distinguishable; reduced motion kills the transforms.
- Search bar: demo plays on load, stops when you type, results read like real hits.
- Block cascade: all three variants visibly differ; caret survives a delete above it.
- Auto-correct: corrections animate in place and the caret never jumps.
Final report: a short list of what you built, any decisions you made beyond this brief, and anything you cut.
