yrs_wrapper — how to do things


CREATE A BOSS
    let boss = BossOfYrs::new(user_id, time);


READ THE WHOLE PAGE
    let blocks = boss.get_entire_page()?;
    Returns all blocks in order.


READ ONE BLOCK
    let text = boss.read_block(block_id)?;
    Returns the block's text. None if no block has that id.


EDIT A BLOCK
    boss.edit_text_block(block_id, edit, target)?;
    The edit is TextEdit::Insert, Delete, or Replace.
    The target is EditTarget::Text or EditTarget::Meta.


DELETE A BLOCK
    boss.delete_block(block_id)?;


SNAPSHOT THE WHOLE DOC
    let snapshot = boss.snapshot()?;


SNAPSHOT JUST THE EDITS
    let bookmark = create_bookmark_of_synced_state(boss.clone())?;
    // ...make edits...
    let diff = generate_diff_snapshot(boss.clone(), bookmark)?;
    The bookmark is the "before edits" marker. Save it, then edit.
    The diff is the edits since then.


MERGE A SNAPSHOT IN
    boss.merge_with_snapshot(bytes)?;
    Takes either a snapshot of just the edits or an entire serialized doc.


MERGE TWO BOSSES
    boss.merge_with(other_boss)?;
    Same as merging a snapshot, without the encode/decode step.


BUILD A BOSS FROM A SNAPSHOT
    let boss = doc_from_snapshot(snapshot, user_id, page_id, time)?;


ACTIVE PAGES AND BACKLINKS
    Both work the same way.
        let pages = YrsActivePages::new_empty();
        let pages = YrsActivePages::new(loaded_from_db)?;
        let links = YrsBacklinks::new_empty();
        let links = YrsBacklinks::new(loaded_from_db)?;
    Both have snapshot() and merge_with_snapshot(bytes) with the same
    meaning as on the boss.


ACTIVE PAGES
    pages.mark_page_active(page_id)?;
    pages.mark_page_deleted(page_id)?;
    let active = pages.is_page_active(page_id)?;
    Returns true if the page isn't in the map at all.


BACKLINKS
    links.set_disabled(bool)?;
    let disabled = links.is_disabled()?;
    Defaults to false if never set.

    And it has the bookmark, same as the boss:
        let bookmark = links.create_bookmark_of_synced_state()?;