# general todo

fix the broken ui wiring in page_edits.rs. it calls the real chain but the view doesn't render.

delete a page.

move a page.

blocks need an ordering key. either main_menu_position on the block row, or a separate table with a map. LocalPages.id is related.

verification for generalized methods that check if stuff is in sync.

the replay system that records events and can replay them back as a json or something.

Callback<()> vs impl Fn() in HappyLittleCheckbox. pick one, and make popup.rs match.

## low priority

session id is faked as "placeholder-session".

user id in create_boss() is hardcoded to "1".


AllCheckBoxStates holds one read signal and one write signal per checkbox, each pair backed by its own localStorage key. Right now it's one pair, placeholder, key placeholder_checkbox. Nothing is unfinished about the struct itself. What's missing is the component being drawn on the page — the HappyLittleCheckbox call is commented out, along with the callback. Also: AllCheckBoxStates holds Signal<bool>, but HappyLittleCheckbox takes ReadSignal<bool>. Those might not be the same type. Check before uncommenting.

inside_page_edits.rs is fully commented out. Component renders a Stylesheet and nothing else. Do this after the main pages work — it's for editing blocks inside a real page.
