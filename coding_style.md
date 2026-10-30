# Coding style

## vocab

**handle** — "this is how you communicate with the actual thing". `router_handle` means "this is the handle to the router", so you call `router_handle.do_something()`, and the handle does the rxtx stuff or whatever is required to communicate with the actual router.

**wrapper** — similar to handle. If there is code that "does something", then `something_doer_wrapper` means "this is what the caller interacts with to abstract away the internals for doing something".

**core** — don't want code inside `lib.rs` or `mod.rs`. If there is no good description for the code, call it `core.rs` and `pub mod` and `pub use` it in `lib.rs` or `mod.rs`.

**main** — the universal word for "this is the entry point for x". If all the db setup is in one file, a good name for that file is `db_main.rs`.

**shared** — if something is shared across multiple things (multiple things own it), prefix it with `shared_`. `shared_list` means "a list that doesn't have a single owner".

**guard** — "do something under certain conditions for important reasons". XGuard sits in front of X. It checks the condition. If the condition holds, X happens. If not, the guard does something else instead — usually refuses or redirects — because the reason matters.

The name says what the guard is related to, not what the condition is. `SetupGuard` is a bad name that's good: if you know setup means MainSetup, then `MainSetupGuard` means "this guard checks for something important related to MainSetup". The condition and what happens on failure live in the code.

**executor** — see "executor" section below.

## caller/client pov

"caller/client pov" isn't a name used in code, but it might show up in docs or comments.

Caller/client pov and internals are completely separate things. Not one thing with hidden parts — separate ways of viewing the same code.

The caller's pov is minimal. It calls `db_handle.do_db_thing()`. That's the whole thing. It doesn't matter to the caller what the handle does inside — whether it holds a tx/rx to the db_wrapper, or a tx/rx to a queue that the wrapper/executor interacts with. To the caller, none of that exists.

When reasoning about a piece of code, it matters which side is meant. Caller pov or internal pov.

## owner + handle

When multiple things want to use something but none of them actually owns it, you split it into an owner and a handle. The owner is a task that holds the real thing. The handle is what everyone else carries around — cheap to clone, passable anywhere, and it lets the caller do things to the real thing without ever touching it directly. The caller's pov is only the handle. What's inside — the channels, the state, the task — is entirely below their awareness.

Main never touches channels. Channels are made inside the constructor, not in main. Main sees only handles. Main makes clear what's happening; actual usage hides internals.

The queue crate doesn't know about the db. That separation of concern matters. The executor is a separate thing, spawned from main with its own handle to the queue — a different kind of handle, one that pulls instead of pushes.

The constructor returns however many handles are needed. One for callers, one for the executor, more if the thing has more than one consumer side. In a real project I've had a constructor return 4 handles.

Channels are the default mechanism for the split. They express "no ownership here, just one thing talking to another", and they sidestep messy async ownership in Rust.

Convention: the constructor is `spawn_task_and_get_handles()` instead of `::new()`, because it tells you what it does — it spawns the owner task and returns the handles. `::new()` hides that.

~~~~
main pov:
  (caller_handle, executor_handle) = spawn_task_and_get_handles()
  spawn executor(executor_handle, db)
  rest_of_program_main(caller_handle)

queue internals pov:
  spawn_task_and_get_handles():
    (push_tx, push_rx) = channel()
    (pull_tx, pull_rx) = channel()
    spawn task_that_owns_the_queue(push_rx, pull_tx)
    return (CallerHandle(push_tx), ExecutorHandle(pull_rx))

handle internals pov:
  pub struct CallerHandle {
    push_tx: mpsc::UnboundedSender<Job>,
  }

  impl CallerHandle {
    pub fn push(&self, job: Job) { self.push_tx.send(job) }
  }

  pub struct ExecutorHandle {
    pull_rx: mpsc::UnboundedReceiver<Job>,
  }

executor pov:
  spawn executor(executor_handle, db):
    loop:
      job = executor_handle.pull_rx.recv()
      run job against db
~~~~

## executor

Executing has been split from something, so ownership isn't one thing anymore.

An executor is a wrapper that requires extra steps from the caller. In cqrs, you gotta build, then you can execute. You can't just execute.

It's a wrapper that's been overcomplicated but for a reason. The other half could be preparing, or a security check, or anything — the point is doing got separated out.

Nothing enforces the split. The name is the signal: somewhere in the code, there exists a helper for generating what the executor expects. Like `db_executor(db_guard.retrieve_permission)` — the guard produces what the executor needs, and the executor doesn't do the checking itself.

## rxtx

AI sucks at names, but it's okay at long names. I love long names.

For rxtx there's a naming rule. The prefix `tx_` or `rx_` is important — it says which direction. The rest of the name is a full sentence: what crosses the channel, and who is on the receiving end. The name has to make it clear: "this handle is so that X can do Y". You shouldn't have to read code to understand what using it does.

Bad: `letters_tx`, `reconnect_rx`. Good: `tx_give_love_letter_that_came_from_server_to_task_that_processes_it`.

## lib.rs and mod.rs

`lib.rs` and `mod.rs` are plumbing only. `pub mod`, `pub use`, `use`, nothing else. Real code goes in a named file next to them, and `lib.rs` / `mod.rs` points at it.

If the code doesn't have a good name, it goes in `core.rs` (see vocab).

## shared_owner/

When a signal (or any state) is created in one place and touched by many others, and you can only look at one file at a time, you can't infer "does anyone do X to this" or "what happens if I do Y".

Convention: a folder called `shared_owner/` holds one file per such thing. The file *is* that thing's shared owner — everyone who touches it finds it there.

What changes is the mechanism. In Leptos, signals have subscribe-and-notify built in, so touching the signal directly is right. In a CLI or server there's nothing reactive, so it's owner + handle + rxtx. Same concept, different mechanism.

What stays the same: the file per thing, in `shared_owner/`, and its doc comment as the map. Every place that touches it: literal path from crate root, why it modifies, when it modifies. Same shape as the naming rules — what would break if this writer didn't exist, why it exists, from a caller pov what do you need to know.

## naming examples

### receive.rs → main_for_receiving_spontaneous_love_letters_from_server.rs

`receive` is a terrible name.

`main` is a word that explains a lot, and it's really powerful here. It basically means you can understand what the thing is without seeing the internals. It doesn't matter if it actually handles it directly or has helpers internally.

It's "main for receiving letters from server". If I made the name myself I'd make it a bit shorter but still long. `main_for_receiving_spontaneous_love_letters_from_server` is essentially the kind of name I'd like AI to produce.

### kick_send.rs → main_send_unsent_love_letters.rs

Internally, the method is just a doorbell. It sends `()` on a channel. The actual sending happens in another file, `send.rs`, which is a task that reads unsent rows and pushes them to the server.

But the caller doesn't know any of that. From the caller's pov, calling this means "I want unsent love letters to be sent." That's the whole deal as far as the caller knows.

Whether it's the actual main method or just a doorbell doesn't matter. `main` is correct because it names the caller's action, so it should be used.

Internals only matter if you need to change internals. At that point you look at the code.

Caller pov names should follow the "caller pov is different from internals pov" guideline. This is that guideline applied.

### send.rs → send_unsent_love_letters_to_server_if_there_are_any.rs

This file is the sender. It's a task that reads unsent rows and pushes them to the server. That's what `main_send_unsent_love_letters` (formerly `kick_send`) rings the bell for.

This one is internal pov, so the name explains the internals directly. It isn't hard to name — a good name is just `send_unsent_love_letters_to_server_if_there_are_any`. Long is fine. It makes it very clear, nothing ambiguous.

### identity.rs → main_for_login_and_registration.rs

This file handles register and login. Picking a name at first run, logging in on reconnect, and what happens when the server says the name is already taken or doesn't exist. It's the entry point for "get this client identified to the server".

Same `main` pattern as `receive`. From the caller's pov, calling into this means "do login/registration things". Whether it handles them directly or has helpers inside doesn't matter.

### folder server_link/ → owner_of_interacting_with_server/

`server_link` is ambiguous. Link could mean anything. This folder owns all the logic related to interacting with the server: opening the socket, registering/logging in the name, sending local changes, receiving remote changes, bootstrapping, and the two-tab guard.

`owner` fits. It's a folder name and the folder owns all the server-interaction logic. From the caller's pov, if you're in this folder, you're interacting with the server. Nothing else in the app talks to the server.

`core.rs` inside it stays `core.rs`. Core is to a folder what `main.rs` is to a crate — the entry point file. It's a convention, not a description.

### merge.rs → main_for_applying_love_letter_from_server_to_app.rs

Takes one love letter and applies it locally. Merges the snapshot into yrs, reconciles the local db mirror, and refreshes the open page if the letter targeted it. If the letter targets a page that isn't open, it just writes to the db.

It isn't "merge with active page" — the active page is only one case. The full job is applying the letter to the app. `main` for that.