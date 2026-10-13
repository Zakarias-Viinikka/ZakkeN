# STORED_AI_CONTEXT — final_test

Durable knowledge for a fresh AI working on this crate. Assumes the workflow file is separate and already known. Assumes the workspace-root `STORED_AI_CONTEXT.md` is available and does not repeat it — this one specializes it.

The crate is the demo that packages the two-page drag-reorder UI behind one reusable component, `PageStack`, so it can be dropped into a real project later. The component knows nothing about databases; the demo app owns the database and feeds the component plain data.

## What this crate is not

`rapier_tests_combined` in the sibling folder is the reference prototype. It is not the same crate: this one adds the boundary, the API, the height model, the ghost model, the sync effect and the cleanup. If the code and either the reference prototype or the task document disagree, the code here wins.

## The API

`PageStack` is the only component exported from `src/page_stack/`. It takes three props, all `#[prop(into)]`: `pages: Signal<Vec<PageInput>>`, `blocks: Signal<Vec<BlockInput>>` and `on_move: Callback<MoveEvent>`. It exports three plain types alongside: `PageInput` (`page_id: String`), `BlockInput` (`block_id`, `page_id`, `content`, all `String`) and `MoveEvent` (`moved_block_id`, `from_page_id`, `to_page_id`, `above: Option<String>`, `below: Option<String>`).

`on_move` fires once per completed drop onto a page, including a drop back into the slot the block came from. It does not fire for a click without movement, for a drag released outside every page, or for a drop into the vacuum. `above` and `below` are the blocks directly before and after the moved block in the destination page's final committed order, never the raw drop-query result. Ghosts count as real neighbours and report their real `block_id`. The callback runs last, after every internal signal has been committed.

## The internal model

Every block gets a stable internal `usize` id from a counter that only goes up. The caller's `block_id` string lives on `Block.block_id: Arc<str>` and is mapped to and from the internal id at the boundary only — one `HashMap<String, usize>` inside the component, plus the reverse lookup by scanning `boxes`.

`Block.content` is an `ArcRwSignal<String>`, not a plain `RwSignal`. The reason is that blocks are created inside an effect, and a plain arena signal created inside an effect gets disposed when the effect reruns. The `Arc` version is owned by the value, not the arena, so it survives.

Page membership is never stored on a block. The world says which page holds a block and in what order; the component derives everything from `rapier.page_order(p)`. Do not add a `page` field to `Block`.

`PAGE_ID_BASE` is `1_000_000_000`. `usize` is 32 bit in wasm and page regions are stored in the same world as block bodies, so the base has to be far above any real block id but still fit in a 32-bit int.

`page_order` sorts by top edge, not by centre. Blocks in a committed layout never overlap, so the order is identical to the centre sort, and a height change can no longer scramble it.

## The ghost model

The vacuum does not remove a block from the layout. It sets `ghost = true` on the block and records its internal id in `stored: RwSignal<Option<usize>>`. The block stays in its page order with the same height, so nothing else moves when a block is vacuumed — the ghost is exactly where the real block was.

Rendering a ghost reuses the same DOM box: `ps-ghost` on the wrapper draws a dashed border and transparent fill, hides the children with `visibility: hidden`, disables the textarea and removes the handle. Because the element is not swapped out, its measured height cannot change when a block becomes a ghost or is restored.

Dragging a block into the vacuum zone (pointer `x <= VACUUM_W + VACUUM_PROXIMITY`): if `stored` already holds a block, clear that block's ghost flag first — it reappears in its own slot, which is the whole "pushed back out" behaviour. Then set `ghost = true` on the dragged block, set `stored` to it. No commit, no world rebuild, no `on_move`. Only the newly ghosted and the newly restored block get confirmation rects.

Dragging the small rect out and dropping it on a page runs `apply_move` on the same internal id with `is_restore = true` and `stored` cleared. The ghost is that block, so nothing needs deleting — the ghost flag is simply cleared. `locate_drop` excludes the moved id, because the ghost is still in the world and would otherwise be returned as its own neighbour. Restore logs "vacuumed" to the console and fires `on_move` normally: from the perspective of the caller, the restore is the move of a block that was sitting in the page its ghost occupied. `from_page_id` is that page.

## The height model

One `ResizeObserver` for the whole component, created once in `PageStack` and stored in `StoredValue::new_local` so the per-block mount effect can capture a `Copy` handle. Every block's box carries a `data-id` attribute with its internal id.

The per-block mount effect auto-sizes the textarea first (height to `auto`, then to `scrollHeight`) and then calls `observer.observe(&el)`. That order matters: it means the observer's first report already has the right height, so no under-then-over flicker and no animation on load.

The observer callback collects `(id, offsetHeight)` pairs, drops heights under 1 and ids no longer in `boxes`, and calls `apply_heights`. `apply_heights` reads every page's order from the world, writes the new heights and `measured = true`, and then commits with `snap_ids = every block`. Snap means no animation: growth from typing resizes the box and every block below it moves to the new position in the same frame. If nothing moved by 0.5 or more and nothing was newly measured, `apply_heights` returns without touching any signal.

Unmeasured blocks use `UNMEASURED_HEIGHT` (56 px) and stay `visibility: hidden` via the `ps-unmeasured` class until their first report. This is what keeps load flicker-free.

The textarea is uncontrolled. Its initial value is set once from the block's `content` signal, and `on:input` writes the text into the signal and re-auto-sizes. Later input changes to the props never overwrite what the user typed — content is initial-value only. Content write-back is a later feature and not in scope here.

## The sync rule

One `Effect::new` tracks only `pages` and `blocks`. Everything else inside it uses `get_untracked`, otherwise the effect would retrigger on every animation frame through `boxes`.

It builds the incoming arrangement: for each page id in order, the block ids of the blocks with that page id, in list order. It builds the current arrangement the same way from the world. If the page ids and both arrangements are equal, it does nothing at all. That is the normal case right after the caller applies a move it just received, and it must not cut animations.

Otherwise it adopts the incoming arrangement: drops blocks whose `block_id` is gone, creates blocks for new ids (next internal id, content signal from the input, estimate height, `measured = false`), keeps existing ones as they are (they keep height, content and ghost flag). It clears `stored` if the stored block is gone, commits with the incoming orders and `snap_ids = every block`, updates the page id list and the page count, rebuilds the world with the new page count, and cancels any active drag and the confirmation overlay.

Blocks whose `page_id` does not match any page in the incoming list are skipped (a console warning is expected in a later revision; the current code drops them silently). Content is initial-value only on creation.

## The Send rule and the compiler traps

`RapierWorld` holds an `Rc`, so it is not `Send`. Leptos 0.8 requires `Send` closures in reactive view positions, in `For` children, and in `Effect::new` bodies. This is the single constraint that most shapes the code here.

The consequence: the world and every `web_sys` handle are touched only in positions that do not require `Send` — event handlers, `Effect::new` bodies, the interval callback and the observer callback. Everything reachable from a reactive view closure is a plain signal read.

When a `Send` closure must reach a non-`Send` handle, store the handle with `StoredValue::new_local` and capture that `Copy` handle instead. The `ResizeObserver` in this crate is the case in point: the per-block mount effect needs the observer, but the effect runs in a `Send` position, so the observer lives in a `StoredValue::new_local` and the effect captures the `Copy` handle and calls `with_value`.

`on_cleanup` on the component clears the interval via its handle, removes both window listeners via `remove()`, and disconnects the observer via `with_value`. The old reference prototype leaked the interval and both listeners with `Closure::forget`; those `forget` calls are gone.

Every non-reactive callback reads through `get_untracked`. The reference prototype called `.get()` inside event handlers and the resulting reactive-context warnings buried the log lines a human has to read.

Per-block style closures read live values from `boxes`, not from the captured `Block`. This is the stale-capture trap documented at the workspace root. A block captured at render time is only the initial state; anything that changes over the block's lifetime (position, ghost flag, measured flag) has to come out of `boxes.get()` inside the closure. Only `id`, `block_id` and the content signal handle are safe to capture per block.

## Environment

The workspace-root `local_cargo.sh` forwarder points at this crate's own `local_cargo.sh`, so `c r` from the root serves `final_test`.

`server_fn_macro` and `server_fn_macro_default` are vendored into `final_test/vendor/`, and the workspace `Cargo.toml` has `[patch.crates-io]` entries pointing at them. The vendored `server_fn_macro_default` uses `syn` "3.0" instead of "2.0" to match the `syn` version leptos pulls in. Removing the patch will bring back the original trait-bound error from `syn` version mismatch. See `SYN_ISSUE_EXPLAINED.md` in the crate.

Trunk builds this crate; bare `cargo` inside the crate does not, because `leptosNew` did not give the crate its own `.cargo/config.toml` and the workspace root's config does not have a wasm target set by default. If a build fails, try `c r` before `cargo`.

## Deviations from the task document

None that the code does not reflect. The `on_move` and vacuum semantics, the ghost model, the height model, the sync rule and the Send rule all match the description in the task document.

## What is not verified

No browser has been opened against this build. The behaviour that has been reasoned out but not seen: no flicker on load, no animation on load, typing growing the box and snapping the blocks below, ghost behaviour matching the description visually, the rect-drag restore landing the block where the drop query says it should, and the single-page checkbox switching cleanly with the vacuum still working. Everything that can be checked at compile time has been.
