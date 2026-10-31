# CQRS and server audit findings

Findings from a three-pass audit by a separate AI, run on 2026-10-31. First pass covered `web_interface/src/`. Second pass added the drag black box, `yrs`, `data_builder_for_operations_that_need_to_be_correct`, `executor_of_what_the_builder_built_because_the_builder_shouldnt_touch_the_db`, and `zakken_server`. Third pass added `text_diff`, `smart_frac_index`, `queue_executor.rs`, `love_letter`, `client_table_blueprints`, the server's remaining plumbing, and the `error_stuff` crates. Later passes supersede earlier ones where they overlap. First-pass output isn't reproduced here — this file is the second and third passes.

**These findings are deliberately left unfixed.** They are the material for the e2e harness — a known list of what the harness is meant to catch. If the harness is good, it fails on the user-visible entries from this list once it's set up. If it doesn't, either the harness has a blind spot or the finding was wrong. Both are useful. Do not act on this file directly; write e2e tests for the visible entries (see the addendum in `e2e_webapp_server_mock.md`) and let them fail until the bugs are fixed on their own schedule.

Findings list, not a plan. No fixes proposed except where one clause points at the cause. First-person "I" below refers to the audit AI, not the human. Items marked "guess" are in code not shown to the audit.

## Corrections to the first pass

**The drag / stale `page.blocks` finding is confirmed, and worse than the first pass said.**

- `on_move` never updates `page.blocks`, and `apply_move` only updates the black box's own boxes and rapier world.
- The next time `page.blocks` changes (any keystroke, split, or remote refresh), `sync_from_props` sees different orders, so the fast paths bail out and `full_rebuild` snaps every block back to the pre-drag order.
- The DB and yrs keep the dragged order, so a reload flips it back again.

That fits the 21 Oct "no room between" errors: splits compute neighbours from the stale order. It also compounds refactor bug #5. The same-page move edits a fresh boss, not the open one. The next flush writes the open boss's full snapshot, which lacks the move, while the mirror keeps the new frac. A later remote letter then makes reconcile emit `UpdateFrac` and silently undo the move locally.

**Menu titles: the first pass was wrong about `MENU_TICK`.**

`apply_blob` returns `changed` from `refresh_open_page_if_matches`, which is false for any page that isn't `page1`. The plan's own `changed` field is ignored. So a remote title edit on a page you haven't just visited never bumps `MENU_TICK`, and the menu doesn't refresh. The `<For>` key issue the first pass raised is secondary to this.

## Check these first

1. **Text offsets are wider than the todo says (the Swedish-text case).** `Doc::new()` uses byte offsets, so `Insert.position` is also affected, not just `Delete` and `Replace`. Three units in play: UTF-16 in the textarea, chars in `chars().count()` and probably `get_diff` (can't see `text_diff`), and bytes in yrs. Any `å/ä/ö` before the edit point can misplace the edit. Belief: yrs panics when an index is out of bounds or mid-codepoint. On wasm that kills the instance, with the write lock possibly held. `apply_edit_to_string` (meta) returns a clean error on bad bounds and the text path doesn't, so the two disagree. Test it.

2. **The black box burns CPU when idle and does quadratic clones on busy frames.**
   - The 16 ms interval calls `boxes.get_untracked()` (a full `Vec` clone) every tick, and `step` clones a snapshot again. Runs even when nothing is moving.
   - Each box's closures (`class_now`, `left_now`, `top_now`, `handle_class` and the ghost signal) call `boxes.get()`, which clones the whole Vec. Every `boxes.set` during a cascade is about 5N² block clones.
   - Fine at 20 blocks, likely visibly janky at a few hundred. Measure it.

3. **A failed commit leaves the transaction open, and later writes silently join it.** `run_in_transaction` does `everything_went_perfectly().await?` with no rollback on failure, and the same goes for `regret_everything().await?`. The next call sees "already in a transaction" and joins the zombie one, so nothing commits until something clears it. Server `accept_letter` has the same gap, but after `begin` has run: on commit failure it returns without `regret_everything`, so every later letter fails at `begin`. Unlikely, but when it happens it's silent data loss.

4. **The server never prunes anything.**
   - `love_letters_from_clients` keeps every letter forever.
   - Unsent rows are queued for every registered user, including abandoned or test names, and never expire.
   - The client's `tbl_seen_server_letters` also grows forever.
   - Each notify makes the server re-push the user's whole unacked backlog, and acks are async. A burst of N letters from another client can push each letter up to N times, which is quadratic.
   - `get_unsent_letter_ids_for_user` has no `ORDER BY`, although a comment claims oldest-first. In practice it's rowid scan order.

5. **The receive path can't tell when yrs parked an update as pending.** A diff whose dependencies haven't arrived (out-of-order delivery, or a lineage mismatch after a server wipe) is accepted without error. The code then reconciles, persists the seen id and acks, so the letter is gone from the server. The plan calls out "parked forever", but nothing checks for it. Unclear whether `encode_diff` includes pending items, so that part needs a test.

6. **Docs and code disagree on same-name devices.** `STORED_AI_CONTEXT` says "same name = same user, second device takes over the socket". The code makes registering an existing name fail with "pick a different one". Server fan-out also excludes the sender by name, so two sockets sharing a name would never receive each other's letters. The "already taken" path calls `clear_user_id`, which leads into the `current_user_id()` panic from the first pass.

## Smaller bugs and edge cases

- **`build_merge_blob` mutates the open boss at build time**, while its header says "Pure" and `CONTEXT.md` says the builder touches nothing. If the executor then fails, the open boss is ahead of the DB. The Fatal path ends the receive task for the session, and the state only heals on reload.
- **`delete_every_block_row` and `ReconcileOp::Delete` key on block id only.** A cross-page move reuses the id and sends two letters, remove on A and add on B. If B's letter is ever applied before A's, the mirror briefly has two rows with that id, and A's delete then removes both. Depends on delivery order, which per item 4 is only incidentally guaranteed.
- **Dead or never-maintained fields:**
  - `is_title` is computed as `idx == 0` but never reconciled, because there's no `UpdateIsTitle` op.
  - The `pages.version` column is written at creation, never updated by `update_page_blob`, and still sent to clients.
  - The exists branch in `apply_bootstrap_row` looks unreachable. `start_over_fresh` already refuses when any page exists, and the receive task is sequential.
- **Same-frac ties.** Two devices splitting the same pair concurrently get identical `key_between` output, and the `(frac, id)` sort handles the tie. The next insert between the tied pair probably errors, since the interval is empty. Can't see `smart_frac_index`, so this is a guess.
- **yrs generates a random client id per `Doc`.** Every `get_yrs_unblobbed`, move and page load edits under a fresh client id. State vectors and blobs accumulate one entry per session over time. Not urgent, but worth knowing.
- **`AtEnd` in yrs uses array order, not highest frac** (known bug #7). Unreachable from the web UI today, since splits use `Between`. The mock client and seeds can still hit it.
- **Server mutex poisoning (guess; can't see `queue_executor.rs`).** A panic inside any handler while holding `state.lock()` poisons it. Every later request then panics on `.unwrap()`.
- **No keepalive on server sockets.** Half-open connections look live to the telephone until a send fails. At-least-once delivery covers the data, but `notify_all` keeps counting them as reached.
- **Doc-level assumptions:** the server has no auth and binds all interfaces. `build_ws_url` picks `wss` on `https` but the server is plain `ws` on `:3000`, so an HTTPS deployment only works behind a proxy.
- **The unflushed-typing window.** `refresh_open_page_if_matches` can overwrite the textarea with merged content while a keystroke typed after the flush is still queued. That diff then applies to different text.
- **The black box is mouse-only.** No touch or pointer events, so it can't drag on a phone browser. Vacuum and ghost feature is still the prototype's. Deliberate for production?

## Half-finished, inconsistent, or stale

**Two ways to do one thing:**
- `create_page` serializes its sketch with `Convert::to_payload()`, and every other builder uses `sketch_to_bytes()`.
- Server `apply_letter` repeats five identical match arms, while the client uses an or-pattern.
- `get_page` and `get_all_pages` duplicate about 40 lines of 5-column destructuring.

**Naming and comments that don't match the code:**
- `mark_page_deleted` survived the delete→disable rename.
- `edit_block_requires_three_db_inserts` actually does two updates and an insert.
- `internal_error` records `file!()` of `errors.rs` rather than the caller's file.
- `send_love_letters` sends one letter.
- The comment in `get_my_unsent_letters.rs` calls the function a "fallback", though the push path itself goes through it.
- `core.rs` ends with a stray `pub use StoredPage`.

**Style guide drift (see `coding_style.md`):**
- `Telephone::new` spawns a task, which is exactly the `::new()` hides it case. `Router::new` does the same.
- The naming pass only touched `web_interface`. The server and builder/executor (`socket.rs`, `core.rs`, `login.rs`) are unnamed in the new style.
- `SocketState.notify` is a bare `rx` name.
- `uniffi::setup_scaffolding!()` in builder `lib.rs` is code in a plumbing file.

## Docs vs code

- `db/README.md` and `HELPER_EXPLAINED.md` describe files that no longer exist (`ui_actions*.rs`, `db_helpers_for_web_client.rs`, `db_out_destruct_helpers.rs`).
- Several docs say `ui_actions/` and `db/reads/`, but the code has `ui_ops/` and `reads_wrapper/`.
- `CURRENT_STATE.md`'s top section is stale (3 tables, 2 sidebar buttons), and the file is a changelog rather than a state doc.
- `STORED_AI_CONTEXT.md` has the "E2E Android testing" section twice.
- `plan_for_general_data_management_ops.md` still uses the pre-rename names (`receive.rs`, `merge.rs`, `identity::login`, `kick_send`, `two_tab::acquire`, `send.rs`) and the removed macro. The e2e plan got a rename note, and this one didn't. A cheap AI will go looking for files that don't exist.

## Already on the list and still true

From `refactor_plan.md`: confirmed bugs #3 (`on_split` passes `false`), #5 (same-page `move_block` loads a fresh boss) and #6 (`existing_blocks` read before `run_exclusive`). #5 matters more now that sync depends on the DB blob, since an open-boss edit can later overwrite a blob that lacks the move.

## Suggested shortlist

Items 1, 3, 5 from "Check these first"; the drag / stale-`page.blocks` bug; and the `changed` fix for menu refresh.

## Third pass (2026-10-31): text_diff, smart_frac_index, queue_executor, love_letter, client_table_blueprints, remaining server plumbing

Most of this batch confirms earlier guesses. But `text_diff` has two real bugs that nothing in the project docs mentions, and one earlier finding turned out to be worse than it looked.

### Confirms and corrections

**Text offsets: confirmed, and worse.** `get_diff` counts chars. The textarea counts UTF-16. yrs defaults to byte offsets. The "Check these first" item #1 only listed `Delete`/`Replace` lengths as the affected positions. `Insert.position` is also in chars and goes straight to yrs, so any multi-byte character before the cursor (å/ä/ö) misplaces inserts too, not just deletes and replaces.

**Same-frac ties.** Now that `key_between` / `key_at_start` / `key_before` / `key_after` are visible — the crate's own logic is consistent, but "sorted as plain strings" only works if the crate's byte encoding is order-preserving. Can't verify without `fracindex` itself (a dependency of `smart_frac_index`, not in the blob). Needs a property test over random `between` / `before` / `after` calls comparing string sort against expected order. Until then it's an assumption.

**Mutex poisoning.** `queue_executor.rs` doesn't have a recovery path. A panic inside any handler while holding `state.lock()` poisons the mutex and every later request panics on `.unwrap()`. Risk is real.

**Cross-page move.** `CreateNewBlockWithId` / `RemoveBlock` do what the first two passes assumed. No correction.

### New: bugs in text_diff

**`get_diff` loses text whenever a selection is replaced by text of a different length.**

It only emits `Insert` or `Delete` when the lengths differ, and the "middle" it extracts comes from a single `diff_amount` taken from the longer string. Concrete case: old `"abXYcd"`, new `"abZcd"` (select `XY`, type `Z`) gives `Delete("XY", 2)`, so the `Z` is never sent. The reverse (select `X`, type `YY`) gives `Insert("YY", 2)` and never deletes the `X`. Equal-length replaces are fine because they get `Replace`.

Anything that replaces a selection or changes length in one input event will hit this: select-and-type, paste over a selection, autocorrect, IME composition, drag-drop. The existing `get_diff` tests don't cover it.

Effect in the app: the UI keeps showing the typed text while yrs doesn't have it. Every later offset in that block is then off, and a reload shows yrs's version, so the text is silently lost.

`DiffResult::Replace` already carries both halves, so the information has a place to go. Confident, from reading the code.

**The `Delete` combine rule has the wrong position semantics.**

`get_diff` outputs are sequential: each one is relative to the text after the previous one. The `Insert` rule respects that (`a_pos + len == b_pos`). The same rule is wrong for deletes.

Example: `del("a", 0)` followed by `del("b", 1)` means "delete char 0, then delete what is now at index 1". That is original chars 0 and 2. Combined into `Delete("ab", 0)`, it removes chars 0 and 1, so the wrong text goes.

Real backspace runs (descending positions) and Delete-key runs (same position) don't match the rule, so they just don't combine. Only the case where the merge is wrong fires. `three_deletes_combine` in the tests encodes the wrong semantics. Reachability is rare (two non-adjacent deletes inside the 500 ms window), but when it hits it corrupts. Confident about the semantics, less about how often it fires.

### New: a rejected letter permanently blocks the client's send queue

Two concrete triggers now visible:

- **Server DB deleted or wiped.** Every queued edit gets "no such page" (`apply_letter` returns `IllegalInput`). The client treats it as `ServerErr`, retries the head row forever, and nothing behind it is ever sent. This is not the same as the earlier "server rejects permanently blocks queue" item from the first pass — that pass described the risk abstractly; now the concrete trigger path is confirmed.
- **Wire-format change.** `client_love_letter` stores `LoveLetterSketch` as raw bincode and the rows outlive app versions. Reordering a variant or field in `LoveLetterSketch`, `TextEdit` or `PositionToInsert` strands those rows. The "frozen order" rule covers `love_letter`'s own enums, but `TextEdit` and `PositionToInsert` live in `yrs` and aren't marked frozen.

### New: smaller bugs and edge cases

- **Hex-string order may not match Fracindex order.** Everything sorts frac positions as plain strings: `(frac, id)` in `yrs`, `ORDER BY frac_position` in sqlite. That is only valid if the crate's byte encoding is order-preserving. Can't see `fracindex`, so this needs a property test over random `between` / `before` / `after` calls.
- **Server-side exit code.** `setup_db` returning `None` makes `main` return with exit code 0. The e2e plan relies on non-zero exit codes to detect a failed start.
- **Server DB is named `test_db.sqlite`** and `setup_db` creates `socket_testing_table` and seeds it with `"text1"` / `"text2"` / `"text3"` in the production DB. Leftover scaffolding.
- **Blocks added to an already-disabled page get `page_is_disabled = "false"`** from `new_every_block_in_existence_row`. A remote edit that lands after a disable letter inserts enabled rows into a disabled page. The menu isn't affected (it only reads title rows), but `get_page_is_disabled` reads one arbitrary row of that page.
- **Client retry after a lost reply** — if the reply is lost, the server stores the same edit as a second letter with a new `letter_id`. Other clients can't dedupe it by id. Only harmless because yrs merges are idempotent.
- **Schema tests don't exist.** The only blueprint tests are inside the commented-out `new_row_helper.rs`, a ~300-line dead file still declared as `pub mod`. `yrs` has no live tests either (`test.rs` and `helper.rs` aren't declared, and `test.rs` targets the old API). `get_foreign_def_*` is never passed to `CreateTableIn`, so there are no foreign keys. There are also no indexes on `my_id_as_given_by_yrs` or `id_of_page_i_belong_to`, which every edit filters on.
- **`seen_server_letters.letter_id` isn't unique.**
- **Queue priority is dead.** `Priority::Important` is never used and every request goes in as `Normal`.
- **Log levels.** Only `log_error` reaches the DB, so `log()` info lines are stdout-only. `Warn` and `CriticalError` are barely used.

### New: dead or stale

- **`text_diff` still carries old UI scaffolding.** `custom_text_area`, `helper::update_diff` and `text_block` pull leptos into the diff crate on wasm. They have a large commented-out block and an unused `node_ref`.
- **`LoveLetterResponse`** is documented as what the server pushes, but the wire uses `UnsentLetter`, and nothing seems to use it.
- **The "msg_id is 0 for spontaneous responses" comment in `main.rs` is stale.** Pushes are now their own `Response::LoveLetterFromServer` variant. `message_id` 0 is only used for the socket's internal pull.
- **`LogCategory::from_str` / `LogLevel::from_str`** shadow the std trait name.
- **Duplicate crate name.** Two different crates are named `error_stuff` (cqrs's and the server's). Given the earlier `db` → `db_core` collision, worth checking that `web_interface` resolves both cleanly.

### Answers to open questions in the plans

**General-data plan, READ FIRST #1 (Telephone kick semantics):**
- Kicking works as designed. `RemoveSocket` drops the entry, so the socket's notify yields `None` and its loop exits. `socket_main`'s own `remove_socket` is a harmless second removal.
- `SocketEntry` has no generation stamp yet, and login still replies plain `ok_serialized()`.

**General-data plan, READ FIRST #3 (server transaction API):**
- The server's `LiveForever` has `begin_all_or_nothing`, `regret_everything` and `everything_went_perfectly`, so a transaction around wipe/adopt is available.

**E2E plan:**
- `log()` is `println!`, which is line-buffered even when piped. The DB fallback in `support/server.py` is therefore unnecessary; stdout is enough.
- `new_liver()` hardcodes `FILEPATH`, as the plan assumed.
- `setup_db` has no path parameter yet.

## Updated suggested shortlist

Priority items, in order:

1. `get_diff` length-changing-selection bug (third pass — real bug, nothing in docs mentions it)
2. `Delete` combine position semantics (third pass — real bug, rare trigger, corrupting)
3. Text offsets, including `Insert.position` (confirmed worse than the todo said)
4. First-run server link never starting (second pass)
5. Drag / stale `page.blocks` (second pass)
6. `apply_blob` returning `changed = false` for non-open pages, so menu titles don't refresh (second pass correction)
7. A failed transaction commit leaving the transaction open (second pass)
8. A rejected letter permanently blocking the send queue (third pass)
