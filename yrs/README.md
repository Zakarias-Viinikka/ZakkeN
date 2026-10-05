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

    Both have these:
        new_empty()
        new(loaded_from_db)          ## takes the blob, returns the unserialized one
        snapshot()                   
        merge_with_snapshot(bytes)   ## applies bytes (whole doc or just edits)
        create_bookmark()            ## the "before edits" marker
        generate_diff_snapshot(bookmark)  ## the edits since the bookmark


ACTIVE PAGES

        mark_page_active(page_id)
        mark_page_deleted(page_id)
        is_page_active(page_id)      ## true if not in the map

    ## the blob in tbl_pages's page_status column goes to new


BACKLINKS

        set_disabled(bool)
        is_disabled()                ## defaults to false