use crate::letter::LoveLetter;
use crate::sketch::LoveLetterSketch;

#[uniffi::export]
pub fn build_a_love_letter(love_letter: LoveLetterSketch, snapshot_of_edit: Vec<u8>) -> LoveLetter {
    match love_letter {
        LoveLetterSketch::EditBlock {
            text_edit,
            edit_target,
            target_page_id,
            block_id,
        } => LoveLetter::EditBlock {
            text_edit,
            edit_target,
            target_page_id,
            block_id,
            snapshot_of_edit,
        },
        LoveLetterSketch::CreateNewBlock {
            position_to_insert,
            target_page_id,
        } => LoveLetter::CreateNewBlock {
            position_to_insert,
            target_page_id,
            snapshot_of_edit,
        },
        LoveLetterSketch::CreateNewBlockWithId {
            block_id,
            position_to_insert,
            target_page_id,
        } => LoveLetter::CreateNewBlockWithId {
            block_id,
            position_to_insert,
            target_page_id,
            snapshot_of_edit,
        },
        LoveLetterSketch::RemoveBlock {
            block_id,
            target_page_id,
        } => LoveLetter::RemoveBlock {
            block_id,
            target_page_id,
            snapshot_of_edit,
        },
        LoveLetterSketch::MoveBlock {
            block_id,
            target_page_id,
            new_frac_position,
        } => LoveLetter::MoveBlock {
            block_id,
            target_page_id,
            new_frac_position,
            snapshot_of_edit,
        },
        LoveLetterSketch::DisablePage { page_id } => LoveLetter::DisablePage {
            page_id,
            snapshot_of_edit,
        },
        LoveLetterSketch::CreateNewPage {
            page_id,
            is_main_menu_page,
        } => LoveLetter::CreateNewPage {
            page_id,
            is_main_menu_page,
            snapshot_of_edit,
        },
    }
}
