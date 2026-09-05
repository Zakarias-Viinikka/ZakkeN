use my_yrs_lib::yrs_wrapper::*;
use std::sync::Arc;

#[test]
fn call_all_methods_and_helpers_without_errors() {
    // Create a fresh document
    let boss = Arc::new(BossOfYrs::new("test_user".to_string()));

    // Insert a block to have something to work with
    Arc::clone(&boss)
        .insert_new_block(
            "hello".to_string(),
            "meta".to_string(),
            PositionToInsert::AtEnd,
        )
        .unwrap();

    // Get a block id for later use
    let block_id = Arc::clone(&boss).get_entire_page().unwrap()[0]
        .id_in_yrs
        .clone();

    // Public methods on BossOfYrs
    let _ = Arc::clone(&boss).get_entire_page().unwrap();
    let _ = Arc::clone(&boss).show_doc_info().unwrap();
    let _ = Arc::clone(&boss).get_user_id().unwrap();
    let _ = Arc::clone(&boss).snapshot().unwrap();

    // edit_text_block_insert (both Text and Meta targets)
    Arc::clone(&boss)
        .edit_text_block_insert(
            block_id.clone(),
            TextEdit::Insert {
                text: " world".to_string(),
                position: 5,
            },
            EditTarget::Text,
        )
        .unwrap();

    Arc::clone(&boss)
        .edit_text_block_insert(
            block_id.clone(),
            TextEdit::Replace {
                old_text: "meta".to_string(),
                new_text: "new meta".to_string(),
                position: 0,
            },
            EditTarget::Meta,
        )
        .unwrap();

    // read_block
    let _ = Arc::clone(&boss).read_block(block_id.clone()).unwrap();

    // merge_with_snapshot
    let other_boss = Arc::new(BossOfYrs::new("test_user".to_string()));
    Arc::clone(&other_boss)
        .insert_new_block("other".to_string(), "".to_string(), PositionToInsert::AtEnd)
        .unwrap();
    let other_snapshot = Arc::clone(&other_boss).snapshot().unwrap();
    Arc::clone(&boss)
        .merge_with_snapshot(other_snapshot)
        .unwrap();

    // merge_with
    let other_boss2 = Arc::new(BossOfYrs::new("test_user".to_string()));
    Arc::clone(&boss).merge_with(other_boss2).unwrap();

    // Free functions
    let bookmark = create_bookmark_of_synced_state(Arc::clone(&boss)).unwrap();
    let _diff = generate_diff_snapshot(Arc::clone(&boss), bookmark).unwrap();

    // doc_from_snapshot
    let snapshot_for_new = Arc::clone(&boss).snapshot().unwrap();
    let _new_boss = doc_from_snapshot(
        snapshot_for_new,
        "test_user".to_string(),
        "test_page".to_string(),
    )
    .unwrap();

    // If we reach here, no errors occurred
}
