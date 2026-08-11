#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(warnings)]
#![allow(unused)]

use crdt_test::yrs_wrapper::{self, *};

fn main() {
    let mut boss_of_yrs = BossOfYrs::new();
    let user_id = 0.to_string();
    let pageid_1 = generate_key(&user_id);
    let pageid_2 = generate_key(&user_id);

    let example_data1 = "mock data".to_string();
    let example_meta_data1 = "type_of_content: title".to_string();
    boss_of_yrs.insert_new_block(example_data1, example_meta_data1, pageid_1.clone());

    let example_data2 = "dock data".to_string();
    let example_meta_data2 = "type_of_content: content".to_string();
    boss_of_yrs.insert_new_block(example_data2, example_meta_data2, pageid_2.clone());

    boss_of_yrs.show_doc_info(pageid_1.clone());
    boss_of_yrs.show_doc_info(pageid_2.clone());
}
