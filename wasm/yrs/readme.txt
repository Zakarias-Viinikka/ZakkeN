so i managed to store meta_data and text_blocks together.

still not clear if yrs will handle them the way i want to where text_blocks use collaborative conflict resolution (using xmlTextRef) and meta_data uses overwrite. but they're grouped together at least.

really hope they'll actually behave how i want them to because otherwise i'll have to rewrite. so that's probably next on the todo list.

after i finish my "read all blocks from page with id x" i have to hyperfocus on simulating conflicting edits to see if they produced my desired behavior.
---
the other big thing is that the caller needs to track what "yrs_block_id" each textblock has, because otherwise whenever i want to edit a textblock i have to loop the entire array until it finds the textblock with the matching "old_text"

this also means that i should only read from yrs when i want to read the entire list or when i want to re-sync because another device has edited something.

otherwise i just push edits and they'll be handled correctly since the caller will hold the "yrs_text_block_id" and hand it over so changes are made in the correct places. similar thing with the javascript thing where i just need to make sure my logic is correct (with tests and whatnot) and then i need to just "trust" stuff is synced.

can still later down the line build stuff that check if everything is as it should. but that's what i gotta work with for now I think.
