use serde::{Deserialize, Serialize};

#[derive(uniffi::Record)]
pub struct Block {
    pub text: String,
    pub metadata: String,
    pub id_in_yrs: String,
}

#[derive(uniffi::Enum, Serialize, Deserialize, Clone, Debug)]
pub enum TextEdit {
    Insert {
        text: String,
        position: u32,
    },
    Delete {
        text: String,
        position: u32,
    },
    Replace {
        old_text: String,
        new_text: String,
        position: u32,
    },
}

#[derive(uniffi::Enum, Serialize, Deserialize, Clone, Debug)]
pub enum EditTarget {
    Text,
    Meta,
}

#[derive(uniffi::Enum, Serialize, Deserialize, Clone, Debug)]
pub enum PositionToInsert {
    AtEnd,
    Between {
        block_id_of_above: Option<String>,
        block_id_of_below: Option<String>,
    },
}
