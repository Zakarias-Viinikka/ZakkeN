# general todo

delete a page.

move a page.

blocks need an ordering key. either main_menu_position on the block row, or a separate table with a map. LocalPages.id is related. (main page ordering)

verification for generalized methods that check if stuff is in sync.

the replay system that records events and can replay them back as a json or something.

inside_page_edits.rs is fully commented out. Component renders a Stylesheet and nothing else. Do this after the main pages work — it's for editing blocks inside a real page.

I should be updating the activePages when i make a new page.

the ctr should be read from localstorage because counting how many titles breaks the moment a title is deleted.
