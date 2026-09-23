Start caller-side, since that's how everything else gets built. So the web UI first, before any backend exists.

First shape: a page that can navigate to other pages. Then buttons on a page that do things to blocks — insert, remove, move — and things to text at hardcoded positions — insert, delete, replace.

The holder struct doesn't exist yet for this. Pretend it doesn't. The buttons call stub do_x methods that take whatever parameters are relevant, plus a callback that gets run to "commit the change". The do_x method itself doesn't touch state. Only the callback does. When the holder shows up later, the only thing that changes is what's inside the callbacks.

Move is not in the intent shapes yet. LoveLetterSketch only has EditBlock, CreateNewBlock, RemoveBlock. So move is either a new variant or it's remove-then-insert behind one button. Decide when it comes up.

The point of doing it this way: every button is a hardcoded action. No typing text by hand. So whatever a button does, a test can do the exact same thing, and they can't drift apart. That's what makes the replay idea work later — a replay is just a string of action words, and a test is that same string.