# Tests to write

E2E tests for the drift-version gating work. Approved after walking through
a set of proposals, one at a time. Order is not priority; order is the
order they were discussed.

Some tests need a way out of lonewolf. Today the only real exit is
reconcile, which isn't built yet. Those tests are marked "deferred".

## 2. enter_lonewolf from every starting status

Setup: A connected.

Do:
- Call enter-lonewolf. Assert status is `Lonewolf`.
- Call it again. Assert still `Lonewolf` (not `LoneWolfAndOffline`).
- Reload. Fire a synthetic "offline" event. Call enter-lonewolf.
  Assert `LoneWolfAndOffline`.
- After the first three, make one edit and confirm nothing reaches the
  server within 2 s.

Record only, don't assert: with the server stopped, enter-lonewolf, then
watch the status over a few seconds and after the server comes back. It
will decay back to Connected once the socket reconnects and the stopped
state is forgotten. That's the deferred session-ready gap, not something
to fix here.

## 4. Outgoing while stopped (deferred: reconcile)

Setup: A and B connected, shared page, block b1 open on both.

Do:
- enter-lonewolf on A.
- A makes 5 edits to b1, one at a time, waiting after each until the
  local queue count rises.
- Confirm the queue has 5 rows, the server received nothing, B sees none
  of the edits.
- Leave lonewolf via reconcile. Wait for the drain.

Assert after drain: each edit appears at the server exactly once, B sees
all 5, queue empty.

## 5. Incoming while stopped (deferred: reconcile)

Setup: A and B connected, shared page P open on both.

Do:
- enter-lonewolf on A.
- B edits P, creates a new page Q and types in it, edits P a few more
  times.
- Confirm A applied none of it: A's local P unchanged, no Q, no
  acked-letter ids recorded, the server still holds A's unsent rows.
- Leave lonewolf via reconcile. Wait for the drain.

Assert after drain: every edit from B appears on A exactly once, Q exists
on A exactly once, the server's unsent rows for A are empty.

## 6. Gate closes mid-drain (deferred: reconcile)

Setup: A and B connected, 30 distinct edits queued on A while stopped.

Do:
- Leave lonewolf. The sender starts draining.
- Partway through, enter-lonewolf again.
- Wait a bit. Leave lonewolf. Let the drain finish.

Assert: B's final page state is identical to a run with the same 30 edits
and no mid-drain flip. No duplicate letters, no lost letters.

## 7. Gate closes between handle_one's check and the apply (deferred: reconcile)

Setup: A and B connected, shared page open on both. A holds the write lock
so the incoming letter's apply has to wait.

Do:
- B sends an edit that will arrive while A is holding the lock.
- Before the lock releases, enter-lonewolf on A.
- Confirm the edit was not applied, not acked, not marked seen.
- Leave lonewolf via reconcile. Wait.

Assert: the edit arrives exactly once, is marked seen once, is no longer
in the server's unsent rows.

## 8. Gate vs bootstrap (deferred: reconcile, expected to fail today)

Setup: server has several pages. A is a fresh profile with an empty local
db, so bootstrap will run on first login.

Do:
- Load A. As soon as status reads Connected, enter-lonewolf (before
  login finishes).
- Confirm A's local pages are empty, so bootstrap didn't half-run.
- After leaving lonewolf, confirm bootstrap ran fully and A's pages
  match the server's.

Both assertions are expected to fail today, because the bootstrap one-shot
flag gets burned even when bootstrap was skipped.

## 9. Storm: typing while receiving

Setup: A and B connected, shared page with b1 and b2 open on both.

Do:
- For ~6 s, A and B both type into the page rapidly. A types into b1,
  B types into b2 and occasionally into b1. A also adds new blocks while
  typing.
- Stop. Wait for both queues and the server to drain. B sends one more
  edit.

Assert:
- A, B and the server hold the same set of blocks in the same order, with
  every token appearing exactly once.
- No duplicate block ids.
- A shows B's last edit within a few seconds (receive side stayed alive).
- No "Data problem" popup.

Do not assert the exact converged text. The point is agreement, not a
specific state.

## 10. Popup-once flag and popup text

Setup: A on a profile with no `should_i_drift_popup` row.

Do:
- Force a mismatch. Read the popup text. Confirm the flag row now reads
  `false`.
- Reload. Confirm no second popup.
- Fix the mismatch. Confirm the flag goes back to `true`.

Assert: exactly one popup per mismatch, flag survives reload, flag resets
on a clean connection.

## 11. Fresh client, no stored drift_version

Setup: a profile that has never logged in against this server.

Do: log in.

Assert: no lonewolf. The server's version gets stored as the client's.

## Skipped / not written

- Reload reliability (the "reload 15 times" test). Catching reload flakiness
  is covered by the other tests that reload. A dedicated test just makes
  the suite slow.
- Browser "offline" event. Planned to be handled by the reconnect loop.
  Write this once the loop exists.

## Update - 2026-11-05

Test 003 (`test_two_clients_after_wipe`) is written and passes. It
exercises the two bugs listed in `todo.md` (wipe + reset + re-register,
empty client receiving pushed pages) as a single scenario.

Remaining:
- The other eight drift-gating tests (2, 4, 5, 6, 7, 8, 9, 10, 11) are
  still on the docket. Most are marked deferred on reconcile.
