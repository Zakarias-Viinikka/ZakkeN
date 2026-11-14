use serde::{Deserialize, Serialize};

use crate::drift_solution::DriftSolution;

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
    /// client -> server: delete every page and every stored love letter.
    WipeDataset,
    /// anyone -> server: what drift version is current, and why did it
    /// change. Exempt from the drift check, because asking is how you
    /// find out you were stale.
    GetCurrentDrift,
    /// client -> server: give me the fingerprint of every page you have,
    /// and the drift version you're on. Exempt from the drift check --
    /// a lonewolf client needs this to compare.
    GetPageFingerprints,
    /// client -> server: give me these pages' full data. Unknown ids are
    /// skipped. Exempt from the drift check.
    GetPagesByIds,
    /// client -> server: apply this batch of love letters under the
    /// claimed target drift version. Exempt from the drift check; the
    /// handler checks `target_drift_version` against the current row
    /// itself. Does not bump the drift version, but does bump the
    /// reconcile version, so two reconcilers can't both succeed off
    /// the same snapshot.
    ReconcileLetters,
    /// client -> server: what is the current reconcile version. Exempt
    /// from the drift check. Bumps only when ReconcileLetters succeeds.
    GetReconcileVersion,
    //web app doesn't have normal os access etc, so instead exporting is a
    // request it makes to the server and the server handles it.
    Export,
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct WipeDatasetPayload {
    pub drift_solution: Option<DriftSolution>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct GetPagesByIdsPayload {
    pub page_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ReconcileLettersPayload {
    /// The drift version the client is reconciling against. Must equal
    /// the server's current drift version, or the whole batch is
    /// refused with DriftVersionMismatch.
    pub target_drift_version: String,
    /// The reconcile version the client last read. Must equal the
    /// current one, or another reconciler beat this client and the
    /// whole batch is refused with DriftVersionMismatch.
    pub target_reconcile_version: String,
    /// Each entry is a `LoveLetter` encoded the same way
    /// `SendLoveLetterPayload::love_letter` is.
    pub letters: Vec<Vec<u8>>,
}
