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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_page_insert_two_blocks() {
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
    }

    fn read_from_yrs_doc() {}

    #[test]
    fn insert_into_block() {}
}
