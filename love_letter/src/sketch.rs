use my_yrs_lib::{EditTarget, TextEdit, yrs_wrapper::PositionToInsert};
use serde::{Deserialize, Serialize};

#[derive(uniffi::Enum, Serialize, Deserialize)]
pub enum LoveLetterSketch {
    EditBlock {
        text_edit: TextEdit,
        edit_target: EditTarget,
        target_page_id: String,
        block_id: String,
    },
    CreateNewBlock {
        position_to_insert: PositionToInsert,
        target_page_id: String,
    },
    CreateNewBlockWithId {
        block_id: String,
        position_to_insert: PositionToInsert,
        target_page_id: String,
    },
    RemoveBlock {
        block_id: String,
        target_page_id: String,
    },
    MoveBlock {
        block_id: String,
        target_page_id: String,
        new_frac_position: String,
    },
    DisablePage {
        page_id: String,
    },
    CreateNewPage {
        page_id: String,
        is_main_menu_page: bool,
    },
}
