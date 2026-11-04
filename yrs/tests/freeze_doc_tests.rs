use std::sync::Arc;

use my_yrs_lib::yrs_wrapper::PositionToInsert;
use my_yrs_lib::{BossOfYrs, EditTarget, TextEdit, freeze_doc, freeze_doc_builder};

#[macro_use]
mod macros;

#[test]
fn frozen_doc_has_same_blocks_as_original() {
    let (boss, block_ids) = make_page!(user: "u", time: "1000", texts: ["a", "b", "c"]);

    edit_block(EditBlock {
        boss: &boss,
        block_id: &block_ids[1],
        new_text: "b edited",
    });

    let frozen = freeze_doc_from(FreezeDocFrom { boss: &boss });

    let result = read_blocks(&boss);
    let expected_result = read_blocks(&frozen);
    assert_eq!(result, expected_result);
}

// ============================================================
// HELPERS
// ============================================================

struct EditBlock<'a> {
    boss: &'a Arc<BossOfYrs>,
    block_id: &'a str,
    new_text: &'a str,
}

fn edit_block(params: EditBlock) {
    let old_text = params
        .boss
        .clone()
        .read_block(params.block_id.to_string())
        .unwrap()
        .unwrap();
    params
        .boss
        .clone()
        .edit_text_block(
            params.block_id.to_string(),
            TextEdit::Replace {
                old_text,
                new_text: params.new_text.to_string(),
                position: 0,
            },
            EditTarget::Text,
        )
        .unwrap();
}

struct FreezeDocFrom<'a> {
    boss: &'a Arc<BossOfYrs>,
}

fn freeze_doc_from(params: FreezeDocFrom) -> Arc<BossOfYrs> {
    let input = freeze_doc_builder(params.boss.clone()).unwrap();
    Arc::new(freeze_doc(input).unwrap())
}

fn read_blocks(boss: &Arc<BossOfYrs>) -> Vec<(String, String, String)> {
    boss.clone()
        .get_entire_page()
        .unwrap()
        .into_iter()
        .map(|b| (b.id_in_yrs, b.text, b.metadata))
        .collect()
}
