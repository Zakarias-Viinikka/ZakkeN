#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(warnings)]
#![allow(unused)]

use crdt_test::yrs_wrapper::{self, *};

fn main() -> Result<(), String> {
    let mut boss_of_yrs = BossOfYrs::new();
    let mut block_counter: u32 = 0;
    let page_id = boss_of_yrs.generate_key();

    boss_of_yrs.insert_new_block("hello".to_string(), "v1".to_string());

    println!("=== before any edits ===");
    print_page(&mut boss_of_yrs);

    let block_id = boss_of_yrs
        .get_entire_page()?
        .first()
        .ok_or("no block found")?
        .id_in_yrs
        .clone();

    let snapshot = boss_of_yrs.snapshot();

    let mut doc_a = doc_from_snapshot(&snapshot)?;
    let mut doc_b = doc_from_snapshot(&snapshot)?;

    println!("=== doc A will append \" from A\" and set meta to \"A wins\" ===");
    println!("=== doc B will append \" from B\" and set meta to \"B wins\" ===");

    doc_a.edit_text_block_insert(
        &block_id,
        TextEdit::Insert(" from A".to_string(), 5),
        EditTarget::Text,
    )?;

    doc_a.edit_text_block_insert(
        &block_id,
        TextEdit::Replace {
            old_text: "v1".to_string(),
            new_text: "A wins".to_string(),
            position: 0,
        },
        EditTarget::Meta,
    )?;

    doc_b.edit_text_block_insert(
        &block_id,
        TextEdit::Insert(" from B".to_string(), 5),
        EditTarget::Text,
    )?;

    doc_b.edit_text_block_insert(
        &block_id,
        TextEdit::Replace {
            old_text: "v1".to_string(),
            new_text: "B wins".to_string(),
            position: 0,
        },
        EditTarget::Meta,
    )?;

    doc_a.merge_with(&doc_b)?;
    doc_b.merge_with(&doc_a)?;

    println!("=== result after merge ===");
    print_page(&mut doc_a);
    print_page(&mut doc_b);

    Ok(())
}

fn print_page(boss_of_yrs: &mut BossOfYrs) {
    match boss_of_yrs.get_entire_page() {
        Ok(blocks) => {
            println!("--- page ---");

            for block in blocks {
                println!(
                    "  id: {}\n  text: {}\n  meta: {}\n",
                    block.id_in_yrs, block.text, block.metadata
                );
            }
        }

        Err(e) => println!("error reading page: {e}"),
    }
}
