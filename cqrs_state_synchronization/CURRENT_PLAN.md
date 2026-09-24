# Purpose

Test keeping the state of the yrs doc and the "friendly" version that's
stored directly in a table in sync. Done in a different project, so the
problem is isolated without having to figure out things that don't matter
at the same time.

A dev friendly UI, not a terminal, so tests can be figured out more easily
and debugged easily. Buttons that seed data or do actions. So tests can be
reasoned about — how they work, how to fix them, how to make new ones.

Later, the web client and the android app use this as the reference for
how to keep the two states in sync reliably.

## This is not a dependency

It's not a dep. It's in its purest form. A template/example, without
forcing really complicated dependency stuff. Unless the methods can be
generalized. That's for future me to figure out.

## The two states

The yrs state needs to match the friendly data version state.

## The UI

A place where buttons can be selected, then buttons for doing stuff.
Like "remove x at y" or "inserting x at y" where stuff is hardcoded.
Replace, or delete, or whatever.

Text is never edited manually. So any way text is manipulated can be
mirrored in actual tests easily.

## Replays

A way to "record" buttons. Do stuff, record it, get a replay, make a
test based off the replay.

A method that takes a string, splits by spaces, turns it into enum
variants. Each enum variant calls a respective method. So a replay can be
created, and a test made from it directly.
## Reminder: HappyLittleCheckbox callback type
I used `impl Fn() + 'static` because I just didn't know Leptos had `Callback<()>`.
Look into `Callback<()>` vs `impl Fn() + 'static` for the `speak_your_truth` param.
Figure out which is idiomatic and whether it matters (ergonomics, Clone, Send+Sync, etc).
