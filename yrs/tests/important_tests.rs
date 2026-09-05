use my_yrs_lib::yrs_wrapper::*;
use std::sync::Arc;

#[cfg(test)]
mod tests {
    use super::*;

    fn new_test_boss() -> Arc<BossOfYrs> {
        Arc::new(BossOfYrs::new("test_user".to_string()))
    }

    fn new_test_doc_from_snapshot(snapshot: Vec<u8>) -> Arc<BossOfYrs> {
        Arc::new(
            doc_from_snapshot(snapshot, "test_user".to_string(), "test_page".to_string()).unwrap(),
        )
    }

    #[test]
    fn create_page_n_insert_two_blocks() {
        let boss = new_test_boss();

        let text1 = "mock data".to_string();
        let text2 = "dock data".to_string();

        let meta1 = "type_of_content: title,".to_string();
        let meta2 = "type_of_content: content,".to_string();

        Arc::clone(&boss)
            .insert_new_block(text1.clone(), meta1.clone())
            .unwrap();
        Arc::clone(&boss)
            .insert_new_block(text2.clone(), meta2.clone())
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
            .insert_new_block("hello".to_string(), "".to_string())
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
            .insert_new_block("".to_string(), "hello".to_string())
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
}
