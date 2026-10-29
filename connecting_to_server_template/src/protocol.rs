//! Protocol types shared between the simulated client and server halves.
//! Mirrors the real `zakken_server` protocol shapes, simplified. Any field
//! the real protocol carries that the template has no use for (version,
//! encoded string flags, etc.) is dropped or replaced with a simpler
//! stand-in.

// PLACEHOLDER: canned page ids used by both the server seed and the local
// placeholder pages. The real app draws these from the db.
pub const CANNED_PAGE_ID_A: &str = "page_a";
pub const CANNED_PAGE_ID_B: &str = "page_b";
pub const CANNED_PAGE_ID_C: &str = "page_c";

#[derive(Clone, Debug)]
pub struct DatasetGenerationPayload {
    pub dataset_generation: i64,
}

#[derive(Clone, Debug)]
pub struct PageRow {
    pub page_id: String,
    pub blobbed_page: Vec<u8>,
    pub page_status: Vec<u8>,
    pub is_main_menu_page: bool,
}

#[derive(Clone, Debug)]
pub struct GetAllPagesPayload {
    pub pages: Vec<PageRow>,
}

#[derive(Clone, Debug)]
pub struct AdoptPage {
    pub page_id: String,
    pub is_main_menu_page: bool,
    pub snapshot: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct UnsentLetter {
    pub letter_id: i64,
    pub love_letter: LoveLetter,
}

#[derive(Clone, Debug)]
pub struct GetMyUnsentLettersPayload {
    pub letters: Vec<UnsentLetter>,
}

#[derive(Clone, Debug)]
pub enum LoveLetter {
    EditBlock {
        target_page_id: String,
        block_id: String,
        text_edit: String,
        snapshot_of_edit: Vec<u8>,
    },
    CreateNewPage {
        page_id: String,
        is_main_menu_page: bool,
        snapshot_of_edit: Vec<u8>,
    },
    DisablePage {
        page_id: String,
        snapshot_of_edit: Vec<u8>,
    },
}

impl LoveLetter {
    pub fn pretend_edit(page_id: impl Into<String>, text: impl Into<String>) -> Self {
        LoveLetter::EditBlock {
            target_page_id: page_id.into(),
            block_id: String::new(),
            text_edit: text.into(),
            snapshot_of_edit: Vec::new(),
        }
    }

    pub fn generic_merge(page_id: impl Into<String>, snapshot_of_edit: Vec<u8>) -> Self {
        LoveLetter::EditBlock {
            target_page_id: page_id.into(),
            block_id: String::new(),
            text_edit: String::new(),
            snapshot_of_edit,
        }
    }
}
