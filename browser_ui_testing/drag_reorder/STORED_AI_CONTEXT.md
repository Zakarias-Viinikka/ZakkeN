# STORED_AI_CONTEXT

Durable knowledge for a fresh AI that already knows the user's workflow and can read the code, but did not work on this repo. This is not the workflow — that file governs and is separate. This is the project-specific stuff the code does not say out loud.

## The workspace shape and why it looks like this

The workspace root has no package of its own. It used to: the original leptos app was the package `drag_reorder` at the root. It was moved into `db_test/` and renamed to package `db_test`, so the root could become a pure container for several sibling apps. Root `Cargo.toml` is only a `[workspace]` block. Members are separate trunk apps, each with its own `index.html`, `css/`, `src/`.

Adding a member means editing the root members list. Missing that is the most common way a new crate silently does not build with the workspace.

`c r` from the root forwards through the root `local_cargo.sh` to whichever member it names. As of last edit that member is `rapier_test_2`. `c b` and `c c` at the root fall through to plain cargo at the workspace root. To serve a different member, either edit the root forwarder or `cd` into the member — every member has its own `local_cargo.sh` except `rapier_collision_test`, which leptosNew did not give one.

## Rapier is never stepped here

This is the single most important architectural fact. In all of these crates, rapier2d is a **query engine**, not a simulator. The world is built out of fixed bodies and is never `step()`ed. All motion — dragging, snapping back, reordering, and the planned velocity work — is DOM-driven: positions live in a leptos signal, styles read from it, and rapier only answers questions like "what is above this rect" or "does this overlap that".

The consequence: rapier does not resolve collisions for you. If a task needs real contacts (bounce, push-apart, stacking), that requires stepping the world and is a genuinely different architecture. None of the four crates does that yet. `rapier_collision_test` is the one where this fork is open.

## The `<For>` stale-capture trap

This bit twice in `rapier_test_2` and the code looks innocent. `<For>` reuses the child view when the key matches, so anything you capture at render time — `let home = b.home;` — goes stale the moment that item's data changes. Styles then read the old value forever.

The rule: style closures and event handlers must read the live value out of the signal, not use the captured one:

    boxes.get().iter().find(|x| x.id == id).map(|x| x.home).unwrap_or(home)

The captured `home` is only a fallback. If you see a captured field used directly inside a closure that outlives the render, that is this bug. Symptoms: data changes but nothing moves visually, and dragging an already-moved thing grabs it from its old spot.

## Patterns inside `rapier_world.rs`

The wrapper maps ids both ways — `id_to_body: HashMap<usize, RigidBodyHandle>` and `collider_to_id: HashMap<ColliderHandle, usize>`. Boxes are inserted as fixed bodies at their home center.

`find_above_and_below(rect, exclude_ids, band)` inflates the rect horizontally by `band` on each side for the overlap test, and classifies above/below by **comparing center Ys**, not by strict vertical separation. This matters: the original used strict separation (`b_max.y <= top`) and silently dropped any box that vertically overlapped the dragged one, which read as "sometimes it ignores boxes". Center-Y comparison resolves the overlap case to a side instead of dropping it.

`band` must stay under half the gap between columns. Columns are 160 apart, so `band` must stay under 80. At 80 or more, a drop between the columns overlaps both bands and you would need a nearest-column tiebreak.

Borrow-checker note: `ColliderSet::insert_with_parent` wants `&mut rigid_bodies` and `&mut colliders` at once, which fails with E0499 if you reach through a struct. Destructure the borrow first:

    let Inner { rigid_bodies, colliders, id_to_body, collider_to_id } = &mut *inner;

`RigidBodyBuilder::translation` wants a `Vec2`. `vector![..]` returns a nalgebra `Matrix`, not a `Vec2`, and will not coerce. Use `Vec2::new(x, y)`.

## The confirmation system

`Rect { x, y, w, h }` plus `enum Confirmation { One(Rect), Many(Vec<Rect>) }`. The enum exists because the user wanted both "just this one" and "this list" expressible without a separate call shape. `FourLines` draws four full-page `position: fixed` lines per rect — top, bottom, left, right — each rect getting a colour from a four-entry palette by index. A new value replaces the old wholesale; there is no incremental update.

The vacuum box is excluded from the above/below query. Its activation is proximity-based (within 40px of its rect), not strict containment, and it is never returned as an above/below neighbour.

There is a `viz_on` checkbox that gates every `confirmation.set(...)` and clears the current confirmation when unchecked.

Click versus drag is separated by a 4px mouse-move threshold. Under 4px of travel is a click; past it is a drag. Clicks confirm the clicked box; drags confirm the neighbours. Getting this threshold wrong is why clicking and dragging would otherwise both fire confirmations.

## `rapier_test_2` specifics

`compact_column` closes the hole left behind in the source column when a box moves to the other column. Without it, a moved box leaves a gap that only closes when something else later lands in that column — the user reported exactly this, and the fix was a compaction pass over both columns inside `drop_into_column`.

The vacuum rectangle uses the full 170x90 box footprint for its drop query, not its 60x80 visual size, so the released box lands where the box would actually fit rather than where the small rect visually sits.

A second box dropped into the vacuum overwrites the first. No stacking.

Six boxes per column overflows the 620-tall stage. Left column reaches y=670, right y=710. Not a bug, just a limit.

## `db_test` specifics

getrandom 0.3 will not compile for wasm32-unknown-unknown with the `wasm_js` feature alone. It also needs the cfg rustflag in `.cargo/config.toml`. That config lives at the workspace root, not in the crate, and is discovered by cargo walking up from the working directory — so it applies to `db_test` even though it is a level above.

`update_hard_copied_z_db_web_output.sh` has a hardcoded `DST` that is one level off after the move into `db_test/`. It should point at `.../drag_reorder/db_test/zdb_web_output`. Not yet fixed.

Every `web_internal_db::db_helper` call is async because it goes through a JS worker. There is no uniffi here — this is pure web.

## Environment gotchas that cost time

`leptosNew` writes a `mkdir -p .cargo` line, and zap's interactive guard eats it: "refusing: mkdir typed at an interactive shell (looks like a zap chain)". So every crate it scaffolds has **no `.cargo/config.toml`** — no `build.target = wasm32-unknown-unknown`, no `-Zshare-generics`. Trunk builds these anyway because trunk passes `--target wasm32-unknown-unknown` explicitly; only a bare `cargo build` inside such a crate is affected.

The same guard fires on a bare `mv` typed in a terminal. Use `/bin/mv`. But the guard is not the only problem with `mv`: zap's `mv` refuses directories outright with `read <path>: Is a directory (os error 21)`. Directory moves have to happen in a real shell with `/bin/mv`.

`.cargo` and `.gitignore` are hidden. `t` and `ls` will not show them unless asked, which makes them easy to forget about during a move.

## Open design for `rapier_collision_test`

The crate is scaffolded and empty. The agreed design: one list of boxes, each with an up arrow, a horizontal rule, and a down arrow inside it. Click the top part to move up, bottom to move down. Destination is current position plus a hardcoded offset in that direction. Collision is "suggestive" — gradual velocity, not a hard stop.

The velocity model the user described: the moving box asks rapier for the closest box in the direction of travel, checks edge proximity, waits out a 0.5 second countdown, then ramps velocity to a cap while touching. On reaching its destination while still in contact, it enters a "keep pushing until nobody is touching me" mode, only in the original direction. Cascades propagate: if box 1 pushes box 2 into box 3, box 2 inherits box 1's pushing mode.

The `wait mode` edge case: a box clicked while a chain is still animating is held in stasis, visually and physically, as if never moved. Its click is a queued "move by N" instruction that executes from wherever it ends up once the running chain settles, with its destination recomputed then, not fixed at click time.

Two sub-questions are deliberately open for guessing, because the user declined to decide them: the proximity threshold that starts the 0.5s countdown (fixed pixels, or touching plus a margin), and the exact exit condition for the pushing mode. Guess, and say what was guessed.

The fork that is still unresolved: whether this crate finally steps the rapier world for real, or stays DOM-driven with hand-written velocity. The user leaned toward query-and-manual-velocity but did not commit. Decide it explicitly before writing code, and say which.
