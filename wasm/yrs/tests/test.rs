use my_yrs_lib::yrs_wrapper::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_page_n_insert_two_blocks() {
        let boss = BossOfYrs::new();

        let text1 = "mock data".to_string();
        let text2 = "dock data".to_string();

        let meta1 = "type_of_content: title,".to_string();
        let meta2 = "type_of_content: content,".to_string();

        boss.insert_new_block(text1.clone(), meta1.clone());
        boss.insert_new_block(text2.clone(), meta2.clone());

        let page = boss.get_entire_page().unwrap();

        assert_eq!(page.len(), 2);
        assert_eq!(page[0].text, text1);
        assert_eq!(page[1].text, text2);
        assert_eq!(page[0].metadata, meta1);
        assert_eq!(page[1].metadata, meta2);
    }

    #[test]
    fn textblocks_collaborative_conflict_resolution() {
        let initial_doc = BossOfYrs::new();
        initial_doc.insert_new_block("hello".to_string(), "".to_string());
        let block_id = initial_doc.get_entire_page().unwrap()[0].id_in_yrs.clone();
        let snapshot = initial_doc.snapshot();

        let offline_doc_1 = doc_from_snapshot(snapshot.clone()).unwrap();
        let offline_doc_2 = doc_from_snapshot(snapshot).unwrap();

        offline_doc_1
            .edit_text_block_insert(
                block_id.clone(),
                TextEdit::Insert {
                    text: " world".to_string(),
                    position: 5,
                },
                EditTarget::Text,
            )
            .unwrap();
        offline_doc_2
            .edit_text_block_insert(
                block_id.clone(),
                TextEdit::Insert {
                    text: "greetings, ".to_string(),
                    position: 0,
                },
                EditTarget::Text,
            )
            .unwrap();

        let edit_1_snapshot = offline_doc_1.snapshot();
        let edit_2_snapshot = offline_doc_2.snapshot();

        initial_doc.merge_with_snapshot(edit_1_snapshot).unwrap();
        initial_doc.merge_with_snapshot(edit_2_snapshot).unwrap();

        let final_synced_page = initial_doc.get_entire_page().unwrap();
        assert_eq!(final_synced_page.len(), 1);
        assert_eq!(final_synced_page[0].text, "greetings, hello world");
    }

    #[test]
    fn metablocks_overwrite_on_conflict() {
        let initial_doc = BossOfYrs::new();
        initial_doc.insert_new_block("".to_string(), "hello".to_string());
        let block_id = initial_doc.get_entire_page().unwrap()[0].id_in_yrs.clone();
        let snapshot = initial_doc.snapshot();

        let offline_doc_1 = doc_from_snapshot(snapshot.clone()).unwrap();
        let offline_doc_2 = doc_from_snapshot(snapshot).unwrap();

        offline_doc_1
            .edit_text_block_insert(
                block_id.clone(),
                TextEdit::Insert {
                    text: " world".to_string(),
                    position: 5,
                },
                EditTarget::Meta,
            )
            .unwrap();
        offline_doc_2
            .edit_text_block_insert(
                block_id.clone(),
                TextEdit::Insert {
                    text: "greetings, ".to_string(),
                    position: 0,
                },
                EditTarget::Meta,
            )
            .unwrap();

        let edit_1_snapshot = offline_doc_1.snapshot();
        let edit_2_snapshot = offline_doc_2.snapshot();

        initial_doc.merge_with_snapshot(edit_1_snapshot).unwrap();
        initial_doc.merge_with_snapshot(edit_2_snapshot).unwrap();

        let final_synced_page = initial_doc.get_entire_page().unwrap();
        let final_metatext = final_synced_page[0].metadata.clone();

        // Overwrite semantics: only one of the edits should win completely.
        let expected_possibilities = ["greetings, hello", "hello world"];
        assert!(
            expected_possibilities.contains(&final_metatext.as_str()),
            "Unexpected meta result: {}",
            final_metatext
        );
    }
}
