use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Request {
    Register,
    Login,
    SendLoveLetter,
    GetAllYrsPages,
    /// how a client acknowledges it's received a love letter response
    Ack,
    /// server -> client: something new happened, go request your unsent
    /// letters with GetMyUnsentLetters.
    Nudge,
    /// client -> server: give me every love letter I haven't received yet.
    GetMyUnsentLetters,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RegisterPayload {
    pub user_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct LoginPayload {
    pub user_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct SendLoveLetterPayload {
    pub love_letter: Vec<u8>,
}

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
pub struct AckPayload {
    pub letter_id: i64,
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