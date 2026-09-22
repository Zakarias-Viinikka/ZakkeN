# cqrs_state_synchronization

## What this is

I need to be able to test keeping the state of the yrs doc and the "friendly"
version that's stored directly in a table in sync. And I need to do it in a
different project, so I can isolate the problem without having to figure out
things that don't matter at the same time.

I also need a dev friendly UI, that isn't like the terminal, so that I can
figure out tests more easily. And so I can also debug tests easily. So I would
need to make methods called by buttons that seed data or do actions. And then
I can easily reason about tests, how they work, how to fix them, and how to
make new ones.

And then later the web client and the android app will use the crate as the
reference for how to keep the two states in sync reliably.

## This is not a dependency

It's not a dep. It's in its purest form. A template/example, without forcing
really complicated dependency stuff. Unless I can generalize methods. That's
for future me to figure out. I don't know how difficult that would be yet.

## The two states

The yrs state needs to match the friendly data version state.

## What's inside

It needs the yrs crate. And potentially the love letter crate, so we can
extend it to also test keeping states in sync across programs. I think that
actually makes sense to do instead of splitting those two.

It needs the blueprint crate, and zdb, and textdiff.

It needs its own schema, for some stuff. It gets every_block_in_existence from
the blueprint crate and stuff. But it also needs its own schema, so that's
separate.

It needs a holder that owns both the yrs doc and the db connection at the same
time. The app has BossOfYrs and LiveForever as two separate things and wires
them by hand each time. This needs one struct that has both, so a test or a
button can say "do this thing" and the thing reaches into both sides.

It needs an error type that wraps the others. yrs returns YrsError, z_db
returns DbError, text_diff has its own. This needs one error that wraps all of
them, so a function that touches all three can return one thing.

The error has two fields. One is the context: "here's the context". The other
is the explanation: "here's what this error probably means, or when it
happens".

## The UI

I basically just need a place where I can select which buttons, and then I get
buttons for doing stuff. Like "remove x at y" or "inserting x at y" where
stuff is hardcoded. Replace, or delete, or whatever.

And then I never edit text manually, so to speak. So any way I manipulate text
can be mirrored in actual tests easily.

## Replays

Ideally I probably build a way to "record" buttons. So I can do stuff, record
it, and then get a replay so I can make a test based off the replay too.

I probably make a method that takes a string. And it splits by spaces. And
then it turns it into enum variants. And each enum variant calls a respective
method. That way I can create a replay, and just create a test from it
directly.
