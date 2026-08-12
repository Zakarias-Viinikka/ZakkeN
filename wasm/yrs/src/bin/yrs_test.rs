#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(warnings)]
#![allow(unused)]
use crdt_test::yrs_wrapper::{self, *};
use yrs::{Array, GetString, Map, MapRef, ReadTxn, Text, Transact, XmlTextRef};

fn main() -> Result<(), String> {
    let mut boss_of_yrs = BossOfYrs::new();
    let mut block_counter: u32 = 0;
    let page_id = boss_of_yrs.generate_key();

    boss_of_yrs.insert_new_block(
        "hello".to_string(),
        "v1".to_string(),
        page_id.clone(),
        &mut block_counter,
    );

    println!("=== before any edits ===");
    print_page(&mut boss_of_yrs, page_id.clone());

    let snapshot = boss_of_yrs.snapshot();
    let mut doc_a = doc_from_snapshot(&snapshot)?;
    let mut doc_b = doc_from_snapshot(&snapshot)?;

    println!("=== doc A will append \" from A\" and set meta to \"A wins\" ===");
    println!("=== doc B will append \" from B\" and set meta to \"B wins\" ===");

    edit_block(&mut doc_a, &page_id, " from A", "A wins")?;
    edit_block(&mut doc_b, &page_id, " from B", "B wins")?;

    doc_a.merge_with(&doc_b)?;
    doc_b.merge_with(&doc_a)?;

    println!("=== result after merge ===");
    print_page(&mut doc_a, page_id.clone());
    print_page(&mut doc_b, page_id.clone());
    Ok(())
}

fn edit_block(
    boss_of_yrs: &mut BossOfYrs,
    page_id: &str,
    text_to_append: &str,
    new_meta: &str,
) -> Result<(), String> {
    let array = boss_of_yrs.doc.get_or_insert_array(page_id.to_string());
    let mut txn = boss_of_yrs.doc.transact_mut();

    let block_map = array
        .get(&txn, 0)
        .ok_or_else(|| format!("edit_block: no block at index 0"))?
        .cast::<MapRef>()
        .map_err(|_| format!("edit_block: failed to cast block to MapRef"))?;

    let text_ref = block_map
        .get(&txn, "text")
        .ok_or_else(|| format!("edit_block: no text field"))?
        .cast::<XmlTextRef>()
        .map_err(|_| format!("edit_block: failed to cast text field to XmlTextRef"))?;

    let len = text_ref.get_string(&txn).len() as u32;
    text_ref.insert(&mut txn, len, text_to_append);
    block_map.insert(&mut txn, "meta", new_meta);
    Ok(())
}

fn print_page(boss_of_yrs: &mut BossOfYrs, page_id: String) {
    match boss_of_yrs.get_entire_page(page_id.clone()) {
        Ok(blocks) => {
            println!("--- page: {page_id} ---");
            for block in blocks {
                println!(
                    "  id: {}\n  text: {}\n  meta: {}\n",
                    block.id_in_yrs, block.text, block.metadata
                );
            }
        }
        Err(e) => println!("error reading page {page_id}: {e}"),
    }
}
