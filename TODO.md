# TODO

In the visual testing folder (browser_ui_testing or whatever I named it):

I want to make a project that uses my zap egui project as a reference for how to do what I want.

Basically I want to create a demo for: "create a menu system for reversing a yrs action and displaying the change in the same way zapgui does"
## planned

mock client

A CLI program. On launch it expects a server to be running, otherwise it exits with an error. It asks the server for every page and deserializes them into a hashmap keyed by page_id, where each entry is a yrs_wrapper that owns that page. It has user-input methods (like "enter x") that pick a random page, do something to a random block, and print what it did. Unlike a real client, it doesn't receive love letters — it just makes a love letter, updates its own yrs_wrapper, and sends the letter immediately. It registers with a hardcoded key.

register system

A user has to register before making requests, and the server stores registrations in a table. The eventual use is so the server knows which clients are missing which love letters, but that's future. The immediate goal is just to have the register system in place so the mock client doesn't need to be rewritten later.

server_love_letter crate

A crate for server-side love letter stuff. For now it holds the schema for the server's letter table, and the parsing of a love letter. If parsing turns out to be shared with the client later, extract it into its own crate.

Deferred until later: retention (delete a letter once every registered client has received it) and backfill (stream un-received letters to a client on connect).
