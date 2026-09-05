use my_yrs_lib::yrs_wrapper::*;
use std::sync::Arc;

#[cfg(test)]
mod tests {

    const USER_ID: &str = "test_user";
    use super::*;

    fn new_test_boss() -> Arc<BossOfYrs> {
        Arc::new(BossOfYrs::new(USER_ID.to_string()))
    }

    fn new_test_doc_from_snapshot(snapshot: Vec<u8>) -> Arc<BossOfYrs> {
        Arc::new(doc_from_snapshot(snapshot, USER_ID.to_string(), "test_page".to_string()).unwrap())
    }

    #[test]
    fn create_page_n_insert_two_blocks() {
        let boss = new_test_boss();

        let text1 = "mock data".to_string();
        let text2 = "dock data".to_string();

        let meta1 = "type_of_content: title,".to_string();
        let meta2 = "type_of_content: content,".to_string();

        Arc::clone(&boss)
            .insert_new_block(text1.clone(), meta1.clone(), PositionToInsert::AtEnd)
            .unwrap();
        Arc::clone(&boss)
            .insert_new_block(text2.clone(), meta2.clone(), PositionToInsert::AtEnd)
            .unwrap();

        let page = Arc::clone(&boss).get_entire_page().unwrap();

        assert_eq!(page.len(), 2);
        assert_eq!(page[0].text, text1);
        assert_eq!(page[1].text, text2);
        assert_eq!(page[0].metadata, meta1);
        assert_eq!(page[1].metadata, meta2);
    }

    #[test]
    fn textblocks_collaborative_conflict_resolution() {
        let initial_doc = new_test_boss();
        Arc::clone(&initial_doc)
            .insert_new_block("hello".to_string(), "".to_string(), PositionToInsert::AtEnd)
            .unwrap();

        let block_id = Arc::clone(&initial_doc).get_entire_page().unwrap()[0]
            .id_in_yrs
            .clone();

        let snapshot = Arc::clone(&initial_doc).snapshot().unwrap();

        let offline_doc_1 = new_test_doc_from_snapshot(snapshot.clone());
        let offline_doc_2 = new_test_doc_from_snapshot(snapshot);

        Arc::clone(&offline_doc_1)
            .edit_text_block_insert(
                block_id.clone(),
                TextEdit::Insert {
                    text: " world".to_string(),
                    position: 5,
                },
                EditTarget::Text,
            )
            .unwrap();

        Arc::clone(&offline_doc_2)
            .edit_text_block_insert(
                block_id.clone(),
                TextEdit::Insert {
                    text: "greetings, ".to_string(),
                    position: 0,
                },
                EditTarget::Text,
            )
            .unwrap();

        let edit_1_snapshot = Arc::clone(&offline_doc_1).snapshot().unwrap();
        let edit_2_snapshot = Arc::clone(&offline_doc_2).snapshot().unwrap();

        Arc::clone(&initial_doc)
            .merge_with_snapshot(edit_1_snapshot)
            .unwrap();
        Arc::clone(&initial_doc)
            .merge_with_snapshot(edit_2_snapshot)
            .unwrap();

        let final_synced_page = Arc::clone(&initial_doc).get_entire_page().unwrap();
        assert_eq!(final_synced_page.len(), 1);
        assert_eq!(final_synced_page[0].text, "greetings, hello world");
    }

    #[test]
    fn metablocks_overwrite_on_conflict() {
        let initial_doc = new_test_boss();
        Arc::clone(&initial_doc)
            .insert_new_block("".to_string(), "hello".to_string(), PositionToInsert::AtEnd)
            .unwrap();

        let block_id = Arc::clone(&initial_doc).get_entire_page().unwrap()[0]
            .id_in_yrs
            .clone();

        let snapshot = Arc::clone(&initial_doc).snapshot().unwrap();

        let offline_doc_1 = new_test_doc_from_snapshot(snapshot.clone());
        let offline_doc_2 = new_test_doc_from_snapshot(snapshot);

        Arc::clone(&offline_doc_1)
            .edit_text_block_insert(
                block_id.clone(),
                TextEdit::Insert {
                    text: " world".to_string(),
                    position: 5,
                },
                EditTarget::Meta,
            )
            .unwrap();

        Arc::clone(&offline_doc_2)
            .edit_text_block_insert(
                block_id.clone(),
                TextEdit::Insert {
                    text: "greetings, ".to_string(),
                    position: 0,
                },
                EditTarget::Meta,
            )
            .unwrap();

        let edit_1_snapshot = Arc::clone(&offline_doc_1).snapshot().unwrap();
        let edit_2_snapshot = Arc::clone(&offline_doc_2).snapshot().unwrap();

        Arc::clone(&initial_doc)
            .merge_with_snapshot(edit_1_snapshot)
            .unwrap();
        Arc::clone(&initial_doc)
            .merge_with_snapshot(edit_2_snapshot)
            .unwrap();

        let final_synced_page = Arc::clone(&initial_doc).get_entire_page().unwrap();
        let final_metatext = final_synced_page[0].metadata.clone();

        let expected_possibilities = ["greetings, hello", "hello world"];
        assert!(
            expected_possibilities.contains(&final_metatext.as_str()),
            "Unexpected meta result: {}",
            final_metatext
        );
    }

    #[test]
    fn insert_block_at_specific_position() {
        let boss = new_test_boss();

        let text_a = "A".to_string();
        let text_b = "B".to_string();
        let text_c = "C".to_string();

        let meta_a = "meta_a".to_string();
        let meta_b = "meta_b".to_string();
        let meta_c = "meta_c".to_string();

        Arc::clone(&boss)
            .insert_new_block(text_a.clone(), meta_a.clone(), PositionToInsert::AtEnd)
            .unwrap();
        Arc::clone(&boss)
            .insert_new_block(text_b.clone(), meta_b.clone(), PositionToInsert::AtEnd)
            .unwrap();

        // Insert C at position 1 (between A and B)
        Arc::clone(&boss)
            .insert_new_block(
                text_c.clone(),
                meta_c.clone(),
                PositionToInsert::SpecificPosition(1),
            )
            .unwrap();

        let page = Arc::clone(&boss).get_entire_page().unwrap();

        let expected_len = 3;
        assert_eq!(page.len(), expected_len);

        let expected_0_text = text_a;
        let expected_0_meta = meta_a;
        assert_eq!(page[0].text, expected_0_text);
        assert_eq!(page[0].metadata, expected_0_meta);

        let expected_1_text = text_c;
        let expected_1_meta = meta_c;
        assert_eq!(page[1].text, expected_1_text);
        assert_eq!(page[1].metadata, expected_1_meta);

        let expected_2_text = text_b;
        let expected_2_meta = meta_b;
        assert_eq!(page[2].text, expected_2_text);
        assert_eq!(page[2].metadata, expected_2_meta);
    }
}
