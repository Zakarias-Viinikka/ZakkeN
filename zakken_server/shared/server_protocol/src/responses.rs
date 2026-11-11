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
    /// The drift version the server is on right now. The socket stamps
    /// this onto each LoveLetterFromServer it pushes to the client, so
    /// a stale client can compare on receive and throw away anything
    /// from a dataset it isn't on.
    pub drift_version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct GetReconcileVersionPayload {
    pub reconcile_version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PageFingerprint {
    pub page_id: String,
    pub fingerprint: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct GetPageFingerprintsPayload {
    /// The drift version the server is currently on. The client keeps
    /// this as its target when it later sends ReconcileLetters.
    pub drift_version: String,
    pub pages: Vec<PageFingerprint>,
}
