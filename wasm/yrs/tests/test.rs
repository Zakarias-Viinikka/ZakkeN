/*
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(warnings)]
#![allow(unused)]

use crdt_test::yrs_wrapper::{self, *};

fn main() {
    let boss_of_yrs = BossOfYrs::new();
    let user_id = 0.to_string();
    let key1 = generate_key(&user_id);
    let key2 = generate_key(&user_id);

    let example_data1 = "mock data".to_string();
    let example_meta_data1 = "type_of_content: title,".to_string();
    boss_of_yrs.insert_new_block(example_data1, example_meta_data1, key1);

    let example_data2 = "dock data".to_string();
    let example_meta_data2 = "type_of_content: content,".to_string();
    boss_of_yrs.insert_new_block(example_data2, example_meta_data2, key2);

    boss_of_yrs.show_doc_info();
}

 */

use my_yrs_lib::yrs_wrapper::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_page_n_insert_two_blocks() {
        let mut boss = BossOfYrs::new();

        let text1 = "mock data".to_string();
        let text2 = "dock data".to_string();

        let meta1 = "type_of_content: title,".to_string();
        let meta2 = "type_of_content: content,".to_string();

        boss.insert_new_block(text1.clone(), meta1.clone());
        boss.insert_new_block(text2.clone(), meta2.clone());

        let page = boss.get_entire_page().unwrap();

        let expected_text_1 = text1;
        let expected_text_2 = text2;

        let expected_meta_1 = meta1;
        let expected_meta_2 = meta2;

        assert_eq!(page.len(), 2);
        assert_eq!(page[0].text, expected_text_1);
        assert_eq!(page[1].text, expected_text_2);
        assert_eq!(page[0].metadata, expected_meta_1);
        assert_eq!(page[1].metadata, expected_meta_2);
    }

    #[test]
    fn textblocks_collaborative_conflict_resolution() {
        let mut initial_doc = BossOfYrs::new();
        let initial_text = "hello".to_string();
        let empty = "".to_string();
        initial_doc.insert_new_block(initial_text, empty);
        let block_id = initial_doc.get_entire_page().unwrap()[0].id_in_yrs.clone();
        let snapshot = initial_doc.snapshot();

        let mut offline_doc_1 = doc_from_snapshot(&snapshot).unwrap();
        let mut offline_doc_2 = doc_from_snapshot(&snapshot).unwrap();

        let doc_1_edit = TextEdit::Insert(" world".to_string(), 5);
        let doc_2_edit = TextEdit::Insert("greetings, ".to_string(), 0);

        offline_doc_1
            .edit_text_block_insert(&block_id, doc_1_edit, EditTarget::Text)
            .unwrap();
        offline_doc_2
            .edit_text_block_insert(&block_id, doc_2_edit, EditTarget::Text)
            .unwrap();

        let edit_1_snapshot = offline_doc_1.snapshot();
        let edit_2_snapshot = offline_doc_2.snapshot();

        initial_doc.merge_with_snapshot(&edit_1_snapshot).unwrap();
        initial_doc.merge_with_snapshot(&edit_2_snapshot).unwrap();

        let final_synced_page = initial_doc.get_entire_page().unwrap();
        assert_eq!(final_synced_page.len(), 1);
        assert_eq!(final_synced_page[0].text, "greetings, hello world");
    }

    #[test]
    fn metablocks_overwrite_on_conflict() {
        let mut initial_doc = BossOfYrs::new();
        let initial_text = "hello".to_string();
        let empty = "".to_string();
        initial_doc.insert_new_block(empty, initial_text.clone());
        let block_id = initial_doc.get_entire_page().unwrap()[0].id_in_yrs.clone();
        let snapshot = initial_doc.snapshot();

        let mut offline_doc_1 = doc_from_snapshot(&snapshot).unwrap();
        let mut offline_doc_2 = doc_from_snapshot(&snapshot).unwrap();

        let insert1 = " world".to_string();
        let insert2 = "greetings, ".to_string();
        let doc_1_edit = TextEdit::Insert(insert1.clone(), 5);
        let doc_2_edit = TextEdit::Insert(insert2.clone(), 0);

        offline_doc_1
            .edit_text_block_insert(&block_id, doc_1_edit, EditTarget::Meta)
            .unwrap();
        offline_doc_2
            .edit_text_block_insert(&block_id, doc_2_edit, EditTarget::Meta)
            .unwrap();

        let edit_1_snapshot = offline_doc_1.snapshot();
        let edit_2_snapshot = offline_doc_2.snapshot();

        initial_doc.merge_with_snapshot(&edit_1_snapshot).unwrap();
        initial_doc.merge_with_snapshot(&edit_2_snapshot).unwrap();

        let final_synced_page = initial_doc.get_entire_page().unwrap();
        let allowed_result_1 = "greetings, hello";
        let allowed_result_2 = "hello world";

        let final_metatext = final_synced_page[0].metadata.clone();

        let expected_meta_result_correct;
        if final_metatext == allowed_result_1 || final_metatext == allowed_result_2 {
            expected_meta_result_correct = true;
        } else {
            expected_meta_result_correct = false;
        }

        println!("original text: {initial_text}");
        println!("insert 1: {insert1}");
        println!("insert 2: {insert2}");
        println!("expected result: '{allowed_result_1}' or '{allowed_result_2}'");
        println!("actual result: {final_metatext}");

        assert_eq!(expected_meta_result_correct, true);
    }
}
