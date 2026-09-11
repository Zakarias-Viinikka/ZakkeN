use my_yrs_lib::{EditTarget, TextEdit, yrs_wrapper::PositionToInsert};
use serde::{Deserialize, Serialize};

uniffi::setup_scaffolding!();

// ## --
// the love letter is just the contract for
// "this is how the client describes an edit so the server knows how to apply it"
// ## --

#[derive(Debug, uniffi::Error)]
pub enum BrokenHeart {
    Error(String),
}

impl std::fmt::Display for BrokenHeart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BrokenHeart::Error(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for BrokenHeart {}

impl From<bincode::Error> for BrokenHeart {
    fn from(err: bincode::Error) -> Self {
        BrokenHeart::Error(err.to_string())
    }
}

#[derive(uniffi::Enum, Serialize, Deserialize)]
pub enum LoveLetterSketch {
    EditBlock {
        text_edit: TextEdit,
        edit_target: EditTarget,
        target_page_id: String,
        block_id: String, //so the client can get a love letter back and know which part of the ui to reload
    },
    CreateNewBlock {
        position_to_insert: PositionToInsert,
        target_page_id: String,
    },
    RemoveBlock {
        position: u32,
        target_page_id: String,
    },
    DisablePage {
        page_id: String,
    },
    CreateNewPage {
        page_id: String,
    },
}

#[derive(uniffi::Enum, Serialize, Deserialize)]
pub enum LoveLetter {
    EditBlock {
        text_edit: TextEdit,
        edit_target: EditTarget,
        target_page_id: String,
        block_id: String, // so the client can get a love letter back and know which part of the ui to reload
        snapshot_of_edit: Vec<u8>,
    },
    CreateNewBlock {
        position_to_insert: PositionToInsert,
        target_page_id: String,
        snapshot_of_edit: Vec<u8>,
    },
    RemoveBlock {
        position: u32,
        target_page_id: String,
        snapshot_of_edit: Vec<u8>,
    },
    DisablePage {
        page_id: String,
        snapshot_of_edit: Vec<u8>,
    },
    CreateNewPage {
        page_id: String,
        snapshot_of_edit: Vec<u8>,
    },
}

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
        LoveLetterSketch::RemoveBlock {
            position,
            target_page_id,
        } => LoveLetter::RemoveBlock {
            position,
            target_page_id,
            snapshot_of_edit,
        },
        LoveLetterSketch::DisablePage { page_id } => LoveLetter::DisablePage {
            page_id,
            snapshot_of_edit,
        },
        LoveLetterSketch::CreateNewPage { page_id } => LoveLetter::CreateNewPage {
            page_id,
            snapshot_of_edit,
        },
    }
}

#[uniffi::export]
pub fn sketch_to_bytes(sketch: &LoveLetterSketch) -> Result<Vec<u8>, BrokenHeart> {
    bincode::serialize(sketch).map_err(Into::into)
}

#[uniffi::export]
pub fn love_letter_to_bytes(letter: &LoveLetter) -> Result<Vec<u8>, BrokenHeart> {
    bincode::serialize(letter).map_err(Into::into)
}

#[uniffi::export]
pub fn deserialize_sketch(bytes: Vec<u8>) -> Result<LoveLetterSketch, BrokenHeart> {
    bincode::deserialize(&bytes).map_err(Into::into)
}

#[uniffi::export]
pub fn deserialize_love_letter(bytes: Vec<u8>) -> Result<LoveLetter, BrokenHeart> {
    bincode::deserialize(&bytes).map_err(Into::into)
}
