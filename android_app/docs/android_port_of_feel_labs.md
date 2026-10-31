# Android port of feel_labs

The plan for a new Android project: a Kotlin + Jetpack Compose port of the web playground `feel_labs`, plus one Rust crate for physics. The design below is decided and the hard problems are solved. Implement it, debug with the principles given, don't re-architect. If you find a real flaw in the plan, say so before deviating.

Reference, READ-ONLY, never edit: `/home/zakke/ProgStuff/ZakkeN/messing_around/feel_labs`. Read its STORED_AI_CONTEXT.md first, then the src file of each lab you port (they hold the data tables, copy, constants). Do not read or reuse the old `android_app/`. No DB, no network.

## Sharpening calls that change or tighten the brief

These were decided up front, before this plan landed:

- Cascade lives in one place. On web, rapier is only a static hit-test index and the stepper is hand-written. So the crate owns layout, the stepper and the index. Block cascade runs on it too (Fall/Spring/Ripple as modes), otherwise you'd have two cascade implementations.
- UniFFI instead of hand-written JNI. The app package name never leaks into Rust symbols, so renaming stays cheap.
- Pages stay alive via a measure-but-don't-place host, not `movableContentOf`.
- Drag lab: pages are stacked vertically and the vacuum strip is dropped.

## Phase 0 - talk

Create no files until the human answers.

- Propose placeholders and say they WILL change: dir `messing_around/feel_labs_android`, package `com.example.feellabs`, app name "feel_labs". Wait for confirmation or "just pick".
- Keep the rename surface tiny: namespace/applicationId, source package dir, rootProject.name. The Rust crate is named `cascade_physics` at `<project>/cascade_physics` and its Kotlin package comes from `uniffi.toml` (`"cascade"`), so renaming the app never touches Rust.
- On the machine, look up current stable AGP, Kotlin, Compose BOM, UniFFI, JNA, cargo-ndk. Don't trust memory. Check rustup android targets and the NDK. Report anything missing.

## Constraints (non-negotiable)

1. No entrance animation when a page opens. It appears already settled. In-page interaction (typing, insert, delete, reflow) may animate.
2. Max 3 pages, all kept composed in memory, so navigation is instant.
3. Kotlin never re-implements the cascade stepper. All three Block cascade modes and Drag reorder run on the Rust crate. If the crate API is awkward, change the crate.
4. Dark only, feel_labs identity.

## Stack

Compose foundation only, no Material3 (custom primitives avoid ripple/shape surprises). No Navigation Compose (default transitions): hand-rolled route state + BackHandler. Single Activity, edge-to-edge, imePadding, minSdk 26. Manifest configChanges covers orientation, screenSize, density, fontScale, uiMode so the Activity (and the kept-alive composition) isn't recreated. Strong skipping on. Per-block text uses state-based BasicTextField + TextFieldState.

## 1. Rust crate cascade_physics

- crate-type cdylib + rlib. Deps: rapier2d (same version as the web Cargo.toml, default features only), uniffi (proc-macro style, no UDL). All Android builds use Rust release optimization, even for Gradle debug (rapier is unusably slow at opt-level 0).
- Units are dp everywhere (web px ~ dp). Constants verbatim: ACCEL 1500, MAX_V 600, DT 0.016, CONTACT 3, TRIGGER = GAP - CONTACT. GAP is config. With web GAP 2, TRIGGER is -1, so the release gate is always true and everything starts together. Keep that, don't "fix" it. Spring: K 380, C 22, settle pos 0.2 / vel 5, max v 1200. Ripple stagger 28 ms per rank.
- Port from drag_reorder_lab: Phase {Idle, Waiting, Moving}, the Move integrator (hard stop at target, vel reset on direction flip), should_release, inherit_vel, release_pass, commit (snap rules, <0.5 snap, unmeasured default height 44), and rapier_world (page_order, locate_drop). Drop web-only bits: content signals, ghost/vacuum.
- Modes (handle setting): Fall = the gravity slide with position-gated release. Spring and Ripple come from block_cascade.rs: semi-implicit Euler spring, retarget keeps velocity. Ripple = Spring + Waiting gated by a timer (rank*28ms, rank = index among blocks that actually move). One phase machine, mode only changes the release rule and integrator.
- Layout is in Rust (commit). Pages stack VERTICALLY: page_top[p+1] = page_top[p] + panel_h[p] + page_gap, panel_h = max(min_page_h, content_bottom + edge_pad). Page regions are y-bands, so locate_drop picks the page by y. Config: block_gap, top_pad, edge_pad, page_gap, min_page_h, block_w, mode.
- step(dt) uses variable dt: clamp to 0.032, sub-step so no sub-step exceeds DT (frame rate independent, smooth at 90/120 Hz). step(0) is a valid flush.
- Rapier is a STATIC spatial index exactly as on web: fixed cuboids at block targets + page regions, AABB scans. Rebuilt lazily (dirty flag) before a query. No dynamics, don't add any.
- Exported object (Mutex inside): new(config), set_mode, set_reduced(bool, makes commits snap everything), add_block(id u64, page, est_height), remove_block(id), set_height(id, h), commit_order(orders: per-page id lists, snap_ids), step(dt) -> {moving, changed: [(id, y)]}, page_rects() -> [(page, y, h)], page_order(page), locate_drop(x, y, exclude) -> Option<(page, above: Option<id>)>.
- set_height semantics: if |diff| < 0.5, no-op. Else update h and re-commit snapping only the blocks AFTER it on that page. Never snap other blocks (that would kill an in-flight cascade). add_block takes an estimated height (one line = theme line height + padding) so the first real measurement is almost always a no-op.
- moving stays true while any block is Waiting or Moving. Changed positions are returned only for blocks whose y changed since the last call (commit snaps surface on the next step).
- Host tests (cargo test): a 54 dp Fall takes about 268 ms; the gate is always true; Ripple start times are staggered; commit snap rules; locate_drop picks the right page/above. This is the "no drift" guarantee.

## 2. Build glue

Gradle Exec task: cargo ndk (arm64-v8a, x86_64) release -> app/src/main/jniLibs, then uniffi-bindgen (a tiny bin target in the crate) generates Kotlin from the built library into app/build/generated/uniffi, registered as a source dir. Wire into preBuild with proper inputs/outputs so unchanged Rust doesn't rebuild. Needs JNA as an aar, plus keep rules if minify is on. Read the generated Kotlin to learn the object lifecycle API; it varies by UniFFI version.

## 3. Theme and primitives

- Copy the :root tokens from styles/app.css into a Kotlin FeelTheme (CompositionLocal). Motion object: easeOut (0.22,1,0.36,1), easeSpring (0.34,1.56,0.64,1), easeFall (0.11,0,0.5,0) as CubicBezierEasing, durations 120/220. Fall duration for distance d dp: sqrt(2d/1500) s. Spring easing overshoots, so clamp alpha to 0..1.
- Primitives: Btn, BtnAccent, Seg, Label (11sp caps, tracking), MockPage (surface-2, 1dp border, accent-line top border, radius 14), MockTitle, MockBlock. Touch targets >= 48dp.
- Reduced motion = Settings.Global.ANIMATOR_DURATION_SCALE == 0, exposed as LocalReducedMotion. Compose tweens respect it. The crate loop doesn't, so call set_reduced.
- Skip backdrop blur (phone lag). Use an opaque panel.

## 4. App skeleton

- Route state in one holder: PageList | Page(id) | LabPicker | Lab(id). Back maps to parent. No transitions anywhere between routes.
- KeepAlive(active: () -> Boolean) is a custom Layout: measures its child with the incoming constraints ALWAYS, reports size 0x0 and places nothing when inactive, reports and places normally when active. `active` is read inside the measure scope, so toggling it relayouts but never recomposes. Inactive subtrees are measured, not placed, so they aren't drawn and take no input. Hidden pages are therefore fully laid out and settled before they're ever shown, and with the same constraints.
- Why not movableContentOf (relocates content, doesn't keep a composition alive). Why not SaveableStateHolder (saves state but destroys the composition, so BasicTextFields get recreated on every navigation, which is the real lag source). Why not Pager (imposes swipe/layout machinery).
- Root = Box { NoteArea inside KeepAlive(route is page list or page); LabArea only when route is a lab route; OverlayHost }. Notes stay composed while you're in the labs. Labs mount and unmount via when(lab) { key(lab) }, so leaving disposes everything.
- Inside NoteArea: PageList is a normal composable shown on the list route. PageHost is one KeepAlive per page (keyed by page id), each with its own scroll state, active = (route is Page(thatId)). Page cap 3, the New page button shows 3/3 and disables. Seed 2 pages from the web mock texts. A new page composes once at creation, opens settled. Clear focus on route change.
- NoteStore (plain Kotlin, held by a ViewModel): pages -> {id, title TextFieldState, blocks SnapshotStateList<BlockModel{id: Long, text TextFieldState}>, a CascadeController}. Global Long id counter. The ViewModel's onCleared closes the controllers' Rust handles.
- Page view = MockPage: title field + CascadeColumn of blocks + a bottom action bar (insert below, delete, back). Block actions are always visible 48dp targets (no hover).
- Editing: Enter inserts a new block below (state-based InputTransformation detecting "\n", splits at the caret). Backspace-at-start merge is best effort via onPreviewKeyEvent. Don't sink time into IME backspace: the explicit delete button is the reliable path.
- Lab picker: grouped list (Typing / Blocks / Pages and chrome), same registry data as web main.rs (label, blurb, group). Lab screen: back, lab-head (title + blurb), then LabScaffold(controls, body). Controls render in a bottom bar for thumb reach. Add a "Labs" entry on the page list.

## 5. Shared pieces

- CascadeController (per page / per lab): owns the Rust handle, a map id -> MutableFloatState (y in dp), and a Channel<Unit>(CONFLATED) of kicks. Every mutation (add, remove, set_height, commit_order) goes straight to the crate, then step(0), applies changed ys, and kicks. runFrames() is a suspend fun: for each kick, loop withFrameNanos { dt from the frame time, first dt 0, clamp; step; write changed ys; break when !moving }. Idle = parked on the channel, zero frames.
- Ownership rule: whoever creates the handle closes it, and the LaunchedEffect only borrows it. Labs: remember { controller } + DisposableEffect(onDispose = cancel, then close, set a closed flag that makes calls no-ops). Notes: the ViewModel owns it, the page composable runs LaunchedEffect(controller) { controller.runFrames() }. Everything is on the main thread, so no races.
- CascadeColumn: a custom Layout. Measures children at (maxWidth, 0..inf), places everything at (0,0), and height comes from a state (page_rects / content bottom) read in the measure policy. Each block applies Modifier.graphicsLayer { translationY = y.floatValue * density } (state read in the layer lambda: zero recomposition and zero relayout per frame) and Modifier.onSizeChanged -> set_height(id, h/density). Block alpha is 0 until its first measure (the web ps-unmeasured), then a 220 ms alpha pop-in only for newly inserted blocks. Children are emitted via for (b) key(b.id) so focus and caret survive reorders.
- Typing growth snaps (set_height snaps the blocks below). Only insert, delete and drop cascade. Known tolerated limitation: onSizeChanged lands one frame late, so there can be a one-frame overlap while a block grows.
- Delete flow: Kotlin marks the block dying and calls remove_block + commit_order immediately. The dying visual stays in Kotlin's list for 140 ms (fade + scale .92, y state frozen at its last value), then is dropped. Insert flow: add_block + commit_order(new order, snap=[new]).
- Modifier.collapse(progress: () -> Float): Modifier.layout scaling the measured height by a progress read in the layout lambda. It's the replacement for the CSS grid 1fr->0fr trick. Spacing must live as padding INSIDE the collapsing element, never Arrangement.spacedBy, or a collapsed row leaves a gap. Clip via graphicsLayer { clip = collapsing } (state read).
- OverlayHost: root-level Box sibling. LocalOverlays gives put(key, content) / remove(key) over a mutableStateMapOf of composable lambdas. An Overlay(visible, content) helper registers in a DisposableEffect (rememberUpdatedState for the lambda) and removes itself on unmount. Used by New page popup, Popups modal + toast stack, Settings modal, so they cover the full screen in root coordinates. If this fights back, fall back to fillMaxSize siblings inside the lab stage with no clip.
- BlockEditor: the one editable block composable (BasicTextField on a TextFieldState, onSplit/onMergeUp/onDelete callbacks), shared by Notes, Block cascade, Drag reorder.

## 6. Labs

Read the matching web src file for data, constants and copy.

- Block transition: TextFieldState input + "Add block". Two preview panes STACKED vertically. Only the new line animates: soft rise (translateY 28dp -> 0 + alpha, 550 ms easeOut) and indent settle (translateX 22 -> -5.5 -> 0, 500 ms, easeSpring). Each line is keyed by index and reads its own text through derivedStateOf. The animation is an Animatable started by LaunchedEffect(generation) and applied in graphicsLayer.
- Search bar: copy the 14-doc corpus, scoring, highlight (AnnotatedString, case-insensitive indexOf), demo script (type, hold, erase, 85 ms tick, stops on focus or typing, Replay). Debounce = LaunchedEffect(query) { delay(140) } (cancellation is the debounce). Port the reconcile logic (leaving rows retained, revived if re-matched, leave_gen guard, enter delay 120 ms when something is leaving) as a pure Kotlin function. Rows in a verticalScroll Column (<= 14 rows, no Lazy), fixed ~340 dp results height. Row enter: rise (translateY 12 + scale .96, spring easing 320 ms) + fade 180 ms, stagger 36 ms. Row exit: fall (easeFall 200 ms) + fade, then Modifier.collapse after 120 ms delay. Reversible via animateFloatAsState used as a State read inside lambdas (fine here, low-frequency booleans). Shimmer bar: infinite transition only composed while searching, drawn in drawBehind.
- Deleting a block: three variants, locked per block at click. A per-block coroutine runs Marked -> Leaving -> removed. Hold the Jobs in a map, and Reset cancels them. Red fade (red mark 400 ms, then fade + scale .85 + translateY 8 over 320 ms ease-in). Strike & slide (line drawn across the text with drawWithContent, 220 ms, then translateX 64 + fade 260 ms easeFall, row collapses with a 120 ms delay). Drop (lift -3 dp + scale 1.015 + elevation for 200 ms, then translateY 40 + rotate 3 deg pivot bottom-left, easeFall, alpha fades in the latter half, row collapses). Drop must not clip while falling. Delete is a visible 48dp button.
- Block cascade: CascadeColumn on the crate, Seg Fall / Spring / Ripple -> set_mode. Per-block insert (+) and delete (x) buttons, Reset recreates the controller. Vary heights through text length.
- Drag reorder: ONE stage, pages stacked vertically (crate page_rects drive the panels; panel height changes tween ~180 ms, low-frequency so animate*AsState is fine). Two pages seeded as on web. No vacuum strip. Each non-title block has a 48 dp-wide left handle ("::") with detectDragGestures that starts immediately. Dragged block translation = y + accumulated drag delta as a state read in graphicsLayer, with zIndex raised once at drag start. Other blocks don't move during the drag (web behavior). On release, probe locate_drop(center x, center y, [dragged]). Kotlin computes the new per-page orders (dragged removed, inserted after `above`, never before a title) and calls commit_order(orders, snap=[dragged]), so the dragged block snaps to its slot and everything else cascades. Optional light haptic on pick-up and drop. Enter-split and merge use BlockEditor like the web.
- New page popup: overlay dialog over a mock workspace. Port apply_edit (prefix/suffix diff giving letter ids) as a pure function, driven by snapshotFlow { state.text } (multi-char edits are fine, the diff handles them). Letters in a FlowRow keyed by id, new letters run a 320 ms shake (Animatable keyframes in LaunchedEffect keyed by letter id). Gradient via TextStyle(brush). IME Done = confirm, back or scrim tap = cancel, FocusRequester + imePadding. Panel in 220 ms (translateY 10, scale .97, alpha). Result line. "New page" reopens.
- Main menu: Seg selects the variant (Underline draw / Nudge + pill / Letter wave), one column. Hover becomes a TAP LATCH: tapping a row latches the effect on it, tapping again unlatches (a press-only effect is too brief to judge). Pen (48 dp) wiggles on tap: Animatable sequence -14, +11, -7, +4, 0 degrees, pivot bottom-left, 520 ms, restarted by relaunching the effect (no remount trick needed). Eye toggles disabled: draw with Canvas, SVG path data via PathParser, upper lid scaleY +1 -> -1 about y=12, pupil scale + alpha out in the first half, 480 ms ease-in-out, progress read in the draw lambda. Row dims to .45 when disabled.
- Popups: ticker = one LaunchedEffect(Unit) frame loop owned by the lab. Virtual clock advances only while not held. Held = a modal is open or a toast is pressed (press-and-hold replaces hover). Toast bars are drawBehind reading (clock - born)/life. Spawn every 2800 ms + jitter table, no spawn while a modal is open, MAX_LIVE 4, kinds and lifetimes and messages and details copied from web. Toast rows use Modifier.collapse for exit (380 ms, consumed 220 ms). Tapping opens the modal with a flight-in from the toast's boundsInRoot (fx, fy, fs relative to the screen centre, 380 ms easeOut). The toast stack is bottom-aligned above the nav bar. Modal body scrolls, log block scrolls horizontally. Escape = back press. Swipe-to-dismiss only if trivial.
- Settings modal: full-screen sheet overlay. Top: horizontally scrollable category tabs with a sliding indicator (Animatable offset in graphicsLayer) plus a dirty dot. Pane swaps with a 10 dp directional slide + fade 220 ms. Footer: dirty chip (pulses when close is refused) + Cancel / Save ("Saved" beat 650 ms then close). Back with unsaved changes pulses instead of closing. Copy all option tables from web. State: ONE mutableStateOf<Value> PER setting (not a list or map signal), dirty count via derivedStateOf. Only the five appearance settings feed the live preview scope. Colors via animateColorAsState are fine. No persistence.
- Running into the wall (do last, keep if it works in ~1 hour): Text(onTextLayout) stores the TextLayoutResult in a state. A LaunchedEffect with snapshotFlow { lineCount }.distinctUntilChanged() detects growth and spawns 3 dots (0/150/300 ms) at the right edge, y = getLineBottom(previous last line), each flying -96 dp, 500 ms, fade in 10% / out the last 25%. NEVER write state from inside onTextLayout (layout-loop risk). Reset the baseline on width change. If the layout APIs fight back, skip the lab and say so.
- Skipped: Page entrance, Idle page, Two-version page, Organic split.

## 7. Performance rules (apply everywhere)

- Read animated or fast state only inside lambdas: graphicsLayer {}, drawBehind {}, layout {}, Modifier.offset { }. Never pass a changing value as a plain parameter (offset(y.dp), alpha(a)): that recomposes every frame.
- Keystroke scoping: text lives in per-block TextFieldState, never mirrored into a data class or a list (the web does this and it's a trap). Page-level lists change only on structure. Title is the same. Height changes reach the crate only via onSizeChanged (fires only when the size actually changed). Derived values use derivedStateOf.
- Don't use animateXAsState on: drag offsets, physics y, toast timers, the search demo, anything changing per frame or per keystroke. Use direct state writes read in lambdas, Animatable + LaunchedEffect, or a withFrameNanos loop. animateXAsState is fine for low-frequency booleans (leaving, disabled, active tab), read as State inside lambdas.
- Tick loops: Block cascade, Popups, Search demo, and the Rust step loop all live in composition-scoped coroutines, so unmounting a lab cancels them. Physics loops park on the channel when idle. Notes pages' loops live with their (hidden but composed) page and are idle unless moving. Handle closing is covered under ownership above.
- Page state: NoteStore in a ViewModel (survives anything the manifest doesn't prevent), the composition kept alive by KeepAlive (BasicTextField creation is what's expensive), no rememberSaveable (no persistence needed).
- Real-app translation (put this in STORED_AI_CONTEXT.md). KEEPS: NoteStore/KeepAlive (become an LRU of 3 hydrated pages), per-block TextFieldState with snapshotFlow-debounced write-back on Dispatchers.IO, CascadeController + crate (ids map to row ids), CascadeColumn, collapse, OverlayHost, theme/primitives, New page dialog. LAB-ONLY: the lab area, search corpus + demo, toast spawner, settings draft/saved model, 3-page cap logic, the drag lab's multi-page stage.

## 8. Gotchas

- Compose doesn't clip graphicsLayer by default, so clip explicitly where wanted.
- The spring easing overshoots: clamp alpha and any non-overshooting property.
- withFrameNanos needs a composition coroutine (a MonotonicFrameClock). viewModelScope has none.
- Key everything that has identity: key(id) in loops, state remembered per id.
- Overlay content must read its own state inside its own lambdas (no stale captures).
- A hidden KeepAlive page re-measures when constraints change (IME opening). Accepted.
- Unit consistency: Rust is dp. Multiply by density only when writing translationY.
- UniFFI objects can be closed: guard every call after dispose.

## 9. Order of work (commit after each; prove the risky bit first)

0. Talk phase.
1. Scaffold + theme + primitives + a hello-world crate wired end to end (Gradle -> .so -> UniFFI Kotlin -> step() called from Compose on an emulator and a device). The toolchain is the riskiest part.
2. Real crate port + cargo tests.
3. KeepAlive, routes, NoteStore, note app with CascadeColumn.
4. Lab shell + picker, then Block transition, Main menu, Deleting block, New page popup.
5. Search bar, Popups, Settings modal.
6. Block cascade, Drag reorder.
7. Running into the wall.
8. Audit.

## Acceptance

Opening any page shows zero motion and zero visible first-frame work. In Layout Inspector, typing in one block does not recompose other blocks or the page, and a cascade/drag runs with no recomposition counts rising per frame. Navigating between pages and back from the labs feels instant on a mid-range device. Leaving Block cascade, Popups or Drag reorder stops all their frame callbacks (confirm there is no recurring work). cargo test passes.

## 10. Docs

Create STORED_AI_CONTEXT.md at the project root, "written by AI for AI", dated sections, the same discipline as the web one: architecture, ownership rules, gotchas hit, and the real-app translation list. Plus a README that describes each lab in plain terms. Don't touch the web project.

## Two things to watch when the cheap AI reports back

- UniFFI + JNA toolchain: If step 1 doesn't work end to end, nothing else matters, so don't let it start porting labs until that does.
- IME backspace-merge: It will try to make it work on soft keyboards. It's explicitly best-effort, and the delete button is the real path.
