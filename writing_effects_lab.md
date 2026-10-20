# writing effects lab

A crate of small visual labs for exploring what makes typing and
editing feel satisfying. Each lab is a separate component in the
same crate.

## block transition

When the user types and the text transitions into a new block -
indentation change, new block opens - something visual happens. The
point of the lab is to figure out what that something is.

## running into the wall

When text wraps to a new line, a small subtle effect makes running
into the right edge feel satisfying. The "wall" is the right edge of
the editor.

## deleting a block

Instead of disappearing immediately, deleted text turns red, then
fades out and pops. The reverse of writing.

## idle page

After a period of no activity, the page grays out slightly. Not a
pulse. Then small things in the background appear and start moving,
like the page comes alive a little. Better if this happens in the
actual background behind the content, not on the page content
itself.

## notes

All labs live in one crate. Each lab is a different component. No
implementation decisions here; this file just records the ideas.
