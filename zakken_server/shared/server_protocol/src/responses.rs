use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PageRow {
    pub page_id: String,
    pub blobbed_page: Vec<u8>,
    pub page_status: Vec<u8>,
    pub version: Vec<u8>,
    pub is_main_menu_page: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct GetAllYrsPagesPayload {
    pub pages: Vec<PageRow>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct UnsentLetter {
    pub letter_id: i64,
    pub love_letter: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct GetMyUnsentLettersPayload {
    pub letters: Vec<UnsentLetter>,
}
