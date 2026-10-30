Plan first, then the prompt for the other AI at the bottom. Two caveats:

Line numbers: your paste has none, so bugs are cited by file and function.
main.rs: I couldn't find the stale crate import you mentioned (see bug 1).
Refactor plan: connecting_to_server_template
Ground rules for the whole pass
File + folder, no mod.rs. A split file keeps its name. Its children live in a same-named folder beside it. The parent file is only pub mod lines, plus its namesake item when it has one.
Spell out submodule paths. Call sites write the full path, e.g. server_link::sending::kick_send, with no re-export shims. The one exception is cool_popup. It is a widget with a four-item public face (CoolPopup, show_popup, show_popup_as, PopupKind), and it re-exports those the way your crate::CustomButton already does.
Statics stay private to their owner. Other files go through the accessor. Where a file needs an untracked read, add a one-line current_* accessor beside the signal, following current_conn_state.
Extractions are verbatim. Statements keep their order. Strings, logs, popups and timings are untouched.
Names. Your names stay except where listed. Three names are held until you answer (see the questions below).
src/
  lib.rs  main.rs  app_main.rs  protocol.rs  client_server_coms.rs  custom_button.rs
  local_pages.rs                                   (new)
  sync_mode.rs     + sync_mode/persistence.rs
  server_link.rs   + server_link/{conn_state, dataset_generation, pending_queue, sending, receiving, restart}.rs
  data_ops.rs      + data_ops/{verb_error, busy}.rs
  reconcile.rs     + reconcile/{plan, session, classify, modal}.rs
  client.rs        + client/{border, server_messages, status_line, panel}.rs
  server.rs        + server/{power, latency, client_messages, panel, border}.rs
  controls.rs      (one file, five components)
  overlays.rs      + overlays/{full_screen_blocker, busy_overlay, reconcile_modal}.rs
  cool_popup.rs    + cool_popup/{style, kind, popup_list, toast, modal, ticker}.rs
css/  sides.scss, button.scss
File by file

Cargo.toml, index.html, main.rs, client_server_coms.rs, custom_button.rs: leave them.

lib.rs: add local_pages. Keep the CustomButton re-export.

protocol.rs: leave it, except for one accessor on LoveLetter that returns the page id a letter targets. It replaces the three-arm match buried in receive_letter's log call, and the log text stays identical.

sync_mode.rs: it mixes the state machine with localStorage string plumbing.

Move storage, read_persisted, reason_key and persist_state into sync_mode/persistence.rs.
Break the nested matches inside read_persisted into mode_from_stored_text and reason_from_stored_text.
Rename reason_key to reason_to_stored_text, because the old name hides that it is the inverse of the parse.
persistence.rs reads the mode through the existing current_mode and current_mode_reason accessors.

Why: reading the state machine shouldn't mean scrolling past string matching.

local_pages.rs (new): it holds read_local_pages_placeholder (from data_ops) and apply_server_pages_locally (from server_link). Both keep their PLACEHOLDER markers. Why: both are local-db stand-ins, so the real db code later replaces exactly one file.

server_link.rs -> folder: today it is a 400-line mix of six concerns. The //! doc drops its last two sentences ("arrive in later steps"). They are already false, and "this file" stops being true anyway.

conn_state.rs holds:
ConnState, the connection signal and its accessors.
require_server and is_lonewolf (no new Lonewolf uses anywhere).
The body of the ConnectionLost arm from client.rs, now go_offline_after_connection_lost.
dataset_generation.rs holds:
The two generation signals and their accessors.
GenerationCheck, check_login_generation and record_new_generation.
A new generations_differ_signal, the derive ClientPanel hand-builds today.
pending_queue.rs holds:
The queue and its count signal.
queue_pretend_edit and clear_pending_letters.
Two one-liners lifted out of kick_send: front_pending_letter and drop_front_pending_letter.
sending.rs holds SENDING, LAST_SEND_FAILED and a kick_send shrunk to guard-and-spawn. The rest is split out:
The loop becomes send_pending_letters_until_blocked.
The gated/connected check, used twice today, becomes sending_is_allowed_now.
The Err arm becomes report_send_failure.
receiving.rs holds:
The seen-set, IGNORED_WHILE_STOPPED and APPLIED_COUNT.
clear_seen_everywhere and clear_seen_set.
pull_unsent.
receive_letter, now three visible branches: ignore while stopped, ack if already seen, apply.
apply_letter_locally, the PLACEHOLDER log, which is the part the real merge replaces.
restart.rs holds RestartOutcome, RESTART_IN_FLIGHT, TEMPLATE_USER_NAME, BOOTSTRAPPED, restart_sync_stack, start_over_fresh and run_restart.
run_restart becomes a flat list of steps: reach_server, log_in, stop_sync_if_dataset_changed, then the existing gated early return, then kick_send, start_over_fresh_on_first_connect and pull_unsent.
Each step returns Result<_, String> carrying today's exact failure text ("the server is offline", "login failed: ...", "bootstrap failed: ...", "pull failed: ...").
run_restart turns Err into RestartOutcome::Failed in one place.

restart_sync_stack and run_restart keep their names.

data_ops.rs -> folder: it mixes an error type, the busy-overlay mechanism and the verbs.

verb_error.rs holds VerbError, its constructors and Display. It also gets announce_verb_error (Refused -> Warning popup, Failed -> Error popup), which controls.rs hand-writes twice.
busy.rs holds the busy signal, dataset_op_busy and with_busy.
data_ops.rs keeps only the verbs, ordered as the buttons appear.
New connect_to_server_and_announce replaces the identical spawn-restart-announce in app_main and the Connect button. announce_connect_outcome becomes private to it.
New refuse_unless_connected(message) replaces the four copy-pasted not-connected guards (reconcile.rs uses it too, with each verb's exact message).
Verbs with success and failure arms (wipe_server, adopt_local, wipe_local_and_pull_from_server) get after_<verb>_succeeded and after_<verb>_failed helpers.
Those helpers still run inside with_busy, so popups keep firing under the overlay as you intend.
pause returns VerbError::Refused instead of a bare String. The popup is the same Warning.
"Adopt my data" and "Wipe server & start from mine" stay both wired to adopt_local.

reconcile.rs -> folder: it holds types, signals, session state and three verbs.

plan.rs holds ConflictChoice, ReconcileDecisions, PageDiffSummary and ReconcilePlan.
session.rs holds ReconcileSession and SESSION. It also holds a new three-case enum (no session / apply already started / can restore) that replaces reconcile_cancel's Option<Option<...>>.
classify.rs holds classify_pages_placeholder with its marker.
modal.rs holds:
The plan and error signals.
clear_modal.
A new open_modal, the two set calls currently inline in the Reconcile button.
apply_decisions_from_modal and cancel_from_modal, the bodies of the two callbacks AppMain builds today.
reconcile.rs keeps the three verbs.
reconcile_apply splits into preconditions, mark_apply_attempted, push_pages_missing_on_server, push_client_side_of_conflicts, disable_pages_on_server, rebuild_local_pages (the PLACEHOLDER lines move there), finish_reconcile and announce_reconcile_result.
The discarded and other counts are captured before the network phases, exactly as now.
reconcile_plan and reconcile_cancel share enter_reconciling_mode and restore_previous_mode.

client.rs -> folder:

client.rs keeps Client, now about fifteen lines. It also takes client_started_signal and its static from server_link. That flag belongs to the Start-client button, not the link.
border.rs holds BorderState and BorderComponent. The two derives currently inline in Client become current_border_state and sync_work_in_flight.
server_messages.rs holds ServerMessageReceiver plus spawn_receive_loop. Its two arms become one-line calls.
status_line.rs holds reason_label and status_line. The latter splits into status_while_reconcile_modal_is_open, status_while_sync_is_stopped and status_while_sync_is_running.
panel.rs holds ClientPanel, which loses its curr_a/gm1/gm2 clone dance by using generations_differ_signal.

server.rs -> folder:

server.rs keeps Server (wiring) and seed_pages.
power.rs holds POWERED and powered_signal.
latency.rs holds the two latency constants and delay. It also gets two helpers for the six hand-rolled reply blocks:
reply_after_delay always sends.
reply_after_delay_if_still_powered sends only if the server is still on, otherwise the reply is dropped, as today.
Each arm keeps the same latency constant and the same helper kind it has now.
client_messages.rs holds the receiver. handle becomes a dispatcher.
The four power-independent messages dispatch directly. Everything else goes through one "drop if powered off" guard, then a second match.
One method per message, named for what the server does: reply_whether_online, power_on, power_off, arm_next_fault, log_client_in, reply_with_all_pages, accept_letter_and_reply, forget_acked_letter, reply_with_unsent_letters, wipe_dataset_and_reply, adopt_dataset_and_reply, queue_pretend_letter, simulate_other_client_wiping.
Two shared steps: take_armed_fault (check-and-clear, four arms) and bump_dataset_generation (three arms).
panel.rs holds ServerReadout and ServerPanel.
border.rs holds the server's BorderComponent.

Why: the 200-line match with eleven copies of the same guard is the worst monolith in the crate.

controls.rs: keep it as one file, but Controls becomes a composition of five components:

ControlsSection (the title + hr wrapper repeated four times, emitting identical DOM).
ClientControls.
StoppedSyncControls.
PopupControls.
ServerControls (which owns next_letter_id).

The button-handler clone pile disappears because each section clones its own handle. spawn_verb and the Reconcile button's hand-written error match both use announce_verb_error. The Reconcile button calls reconcile::modal::open_modal.

app_main.rs: AppMain becomes: make the coms, turn_server_on_when_started, connect_once_when_client_starts (both private functions here), then the view!. The modal callbacks come from reconcile::modal. ClientFallback and ServerFallback: leave them.

overlays.rs -> folder: one file per component, named like its CSS file.

ReconcileModalBody splits into ReconcileSummary, ChooseForAllButtons and ConflictRow components.
choose_for_every_conflict and every_conflict_has_a_choice become named functions.
The include_str! paths gain a ../ (they are relative to the source file).

cool_popup.rs: leave it as one file. It is a black box: the only thing callers see is show_popup, show_popup_as, CoolPopup and PopupKind. Splitting it would break that property and spread the widget's private state across several files. This is a deliberate deviation from the earlier plan.

CSS:

Move .btn:disabled from sides.scss to button.scss.
Group the .border-* state rules in sides.scss.
Delete .border-paused, the SyncMode-collapse debris. Strike this if you'd rather it stay as a note.

Names I need you to rule on (left untouched until answered)

BorderComponent is defined twice, in client.rs (state + in_flight) and in server.rs (online). If it's AI-made, I'd call them ClientBorder and ServerBorder.
ClientMessageReceiver (in server.rs, receives MessageToServer), ServerMessageReceiver (in client.rs), and their handle methods. ServerMessageReceiver also has a field called handle. If AI-made, I'd use ServerInbox and ClientInbox with a method receive.
spawn_verb (controls.rs) doesn't say it pops the error. If AI-made, I'd call it spawn_verb_and_popup_its_error.

Behaviour ideas (out of scope)

Server fault replies fire even if power dropped during the delay, while success replies don't. Make them consistent.
The auto-connect effect skips the "already connected/connecting" guards the Connect button has. Put them in the verb.
Disable the Reconcile, Adopt and Wipe buttons while not connected, instead of refusing by popup.
ClientPanel prints Debug forms (Connected, Some(3)). Humanize them.
Status line says "(offline)" while the connection state is Connecting.
Give SENDING a single writer.

Bugs spotted (not fixed)

main.rs: I can't see the stale crate import. The only crate import is connecting_to_server_template::app_main::AppMain, which matches Cargo.toml's package name. The only stale reference I see is the commented-out page1::Page1 route. Tell me if you meant something else, or if the file on disk differs from this paste.
server_link run_restart: BOOTSTRAPPED flips to true before start_over_fresh succeeds, so a failed bootstrap is never retried this page load.
server_link run_restart and client ServerMessageReceiver:
Bootstrap and pull failures return Failed but leave ConnState::Connected. The status line can then say "In sync." right after "Couldn't connect: pull failed".
ConnectionLost is dropped while a restart is in flight, so a power-off after login is never recorded.
server_link kick_send and the ConnectionLost handler: both write SENDING. A reconnect inside the window between them could start two send loops and send the same letter twice.
reconcile reconcile_apply: pushed pages always go out with is_main_menu_page: false, so the server row loses the flag.
reconcile reconcile_plan failure and reconcile_cancel restore: if the previous mode was NeedsReconcile with no stored reason (possible after a reload), the reason stays "reconciling".
app_main on_cancel: the result of reconcile_cancel is ignored and clear_modal always runs. If with_busy refuses, the modal closes but the session stays, and the next Reconcile says "already in progress". Keyboard focus can reach Cancel under the overlay.
Stale comments:
check_login_generation says mismatch is "unreachable here", but "Other client wipes" reaches it. The #[allow(dead_code)] on Changed is stale too.
clear_seen_everywhere says it's a no-op, but it calls clear_seen_set.
busy_overlay.css and reconcile_modal.css claim z-index 200 and 300. Both render inside .fsb-blocker (100) and stack by DOM order.
cool_popup render_modal: the header prints the kind label twice (title and badge).
Unused:
The started prop on Server.
The server_coms prop on Controls.
require_server.
stored_generation().
clear_mode_reason.
GetMyUnsentLettersPayload.
.container in global.scss.
is_lonewolf is yours, so I'm leaving it.
sides.scss: .btn:disabled is in the wrong file (handled in the plan).

---

Prompt for the other AI

Paste this, then paste the approved plan underneath it.

You are performing a pure structural refactor of a Leptos 0.8 CSR crate at
/home/zakke/ProgStuff/ZakkeN/connecting_to_server_template. The approved plan
is pasted below this prompt. Implement it as written. Do not redesign it, and
do not improve anything it doesn't mention.

HARD RULES
- Same behaviour. Statement order, side-effect order, error strings, popup
  text and popup kinds, log lines, latencies, and DOM structure (element
  order, class names) must be identical. If a change would alter any of
  those, don't make it; list it under "behaviour changes I avoided" at the end.
- Extractions are verbatim moves. Where a function reads a value before an
  await (queue counts, `was_stopped`, `queued_before`, `discarded`), keep that
  read at the same point.
- Popups fired inside `with_busy` must still fire inside it after extraction.
- No new dependencies, no new tests, no new features, no new comments. The only
  comment edits allowed are the ones the plan lists, or fixing a comment that
  your move made false. Keep every `// PLACEHOLDER:` and `// STAND-IN:` marker
  with its function.
- Bugs and stale comments listed in the plan must be preserved, not fixed.
- Do not touch Cargo.toml, index.html, main.rs, client_server_coms.rs or
  custom_button.rs.
- Held names: `BorderComponent` (both copies), `ClientMessageReceiver`,
  `ServerMessageReceiver`, their `handle` methods, and `spawn_verb` keep their
  current names. Do not rename them on your own.
- New function names: use the ones in the plan. If you need a name the plan
  doesn't give, make it a literal description of what the function does
  (never handle_x, process_x, do_x, helper).
- Module layout: a split file keeps its name and its children go in a same-named
  folder. No mod.rs. No glob re-exports. Call sites use full submodule paths.
  The only re-exports are in cool_popup.rs (CoolPopup, show_popup,
  show_popup_as, PopupKind).
- Statics stay private to the file that owns them. Other files use the
  accessors. Where a file needs an untracked read, add a one-line `current_*`
  accessor beside the signal, in the style of `current_conn_state`.

GOTCHAS
- `include_str!` paths are relative to the source file. Files moved into
  src/overlays/ need "../../css/<name>.css".
- Extracting an inline `view!` chunk into a component must not add a wrapper
  element. The rendered DOM must match what it was before.
- In the server dispatcher, a message dropped while powered off must be dropped
  synchronously with the reply sender unused, exactly as today (the client
  sees "reply dropped").
- The server's "reply if still powered" and "reply unconditionally" cases are
  different today; keep each arm on the same kind it uses now.

ORDER OF WORK (each step must pass `cargo check` before you start the next;
if the host target can't build web-sys code, use
`cargo check --target wasm32-unknown-unknown`; update every call site
crate-wide at each step; do not commit)
 1. lib.rs, local_pages.rs, the protocol accessor
 2. sync_mode + persistence
 3. server_link split (including moving client_started_signal into client.rs)
 4. data_ops split
 5. reconcile split
 6. client split
 7. server split
 8. overlays split
 9. (skipped — cool_popup stays one file)
10. controls.rs, app_main.rs
11. CSS edits
At the end run `trunk build` if it is available.

IF STUCK
If the plan is ambiguous, or a step cannot be done without changing a signature
or behaviour the plan doesn't mention, stop that step and ask. Do not improvise.

WHEN DONE, REPLY WITH
1. A table of old path -> new path for every item that changed module or name.
2. Any "behaviour changes I avoided".
3. Any new bug you noticed (file and function only). Do not fix it.
