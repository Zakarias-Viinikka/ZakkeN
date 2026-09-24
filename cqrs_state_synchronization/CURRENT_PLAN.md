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

delete_page in page_edits.rs takes RwSignal<i32>, but everything else in that file uses usize. Inconsistent.

for_leptos! is defined twice — once in macros.rs, once at the top of popup.rs. menu.rs uses neither; it uses <For> directly.

The For loop in menu.rs is wrong. It keys by title, and new pages get pushed with an empty title, so multiple entries share the same key. Need to write a fresh <For> leptos starter example, then fix the id being used properly in the actual project.

blueprint crate should export a const for every table name and every column name. even though it's manual, any consumer of the schema just has to figure out which const is correct instead of being unsure whether they copied the right name.

create_new_page in the web_interface page_edits.rs pushes the new page into local_pages before the page actually exists. The id it uses comes from create_page in the builder crate, which hasn't run yet, so the entry gets saved with an empty yrs_id. The push needs to happen inside the spawned block, after create_page returns.
