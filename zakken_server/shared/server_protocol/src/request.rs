use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Request {
    Register,
    Login,
    SendLoveLetter,
    GetAllYrsPages,
    /// how a client acknowledges it's received a love letter
    Ack,
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
pub struct AckPayload {
    pub letter_id: i64,
}