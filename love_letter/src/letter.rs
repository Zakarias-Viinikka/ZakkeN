use my_yrs_lib::{EditTarget, TextEdit, yrs_wrapper::PositionToInsert};
use serde::{Deserialize, Serialize};

#[derive(uniffi::Enum, Serialize, Deserialize)]
pub enum LoveLetter {
    EditBlock {
        text_edit: TextEdit,
        edit_target: EditTarget,
        target_page_id: String,
        block_id: String,
        snapshot_of_edit: Vec<u8>,
    },
    CreateNewBlock {
        position_to_insert: PositionToInsert,
        target_page_id: String,
        snapshot_of_edit: Vec<u8>,
    },
    CreateNewBlockWithId {
        block_id: String,
        position_to_insert: PositionToInsert,
        target_page_id: String,
        snapshot_of_edit: Vec<u8>,
    },
    RemoveBlock {
        block_id: String,
        target_page_id: String,
        snapshot_of_edit: Vec<u8>,
    },
    MoveBlock {
        block_id: String,
        target_page_id: String,
        new_frac_position: String,
        snapshot_of_edit: Vec<u8>,
    },
    DisablePage {
        page_id: String,
        snapshot_of_edit: Vec<u8>,
    },
    CreateNewPage {
        page_id: String,
        is_main_menu_page: bool,
        snapshot_of_edit: Vec<u8>,
    },
}

#[derive(uniffi::Record, Serialize, Deserialize)]
pub struct LoveLetterResponse {
    pub love_letter: LoveLetter,
    /// this is the id the client uses to keep track of whether it's received a love letter twice.
    /// this can happen if it receives a love letter, the connection dies, and then it reconnects,
    /// the server would send the love letter again since it never received an acknowledgment since server crashed.
    /// alternative would be to wrap the entire thing in a transaction client side, but that means
    /// every love letter starts a transaction and blocks other transactions, which is a problem
    /// since every edit to a page is a transaction.
    pub letter_id: i64,
}
