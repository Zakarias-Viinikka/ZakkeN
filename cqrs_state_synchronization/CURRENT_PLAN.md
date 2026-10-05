## Replays

A way to "record" buttons. Do stuff, record it, get a replay, make a
test based off the replay.

A method that takes a string, splits by spaces, turns it into enum
variants. Each enum variant calls a respective method. So a replay can be
created, and a test made from it directly.

delete_page in page_edits.rs takes RwSignal<i32>, but everything else in that file uses usize. Inconsistent.
