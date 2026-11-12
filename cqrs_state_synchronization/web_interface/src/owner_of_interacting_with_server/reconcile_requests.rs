//! Caller pov: async functions for the server's reconcile-side endpoints.
//! Each builds a `ClientCommand`, sends it through `router_handle::send`,
//! decodes the bincode `Result<T, ServErr>` payload, and hands back the
//! inner Result.
//!
//! Nothing here touches CONN_STATE or `sync_may_run`. These requests are
//! exempt from the drift check on the server, so a lonewolf client can
//! call them. `send` itself refuses if the socket isn't open.

use protocol::serialization::Convert;
use server_protocol::{
    GetAllYrsPagesPayload, GetPageFingerprintsPayload, GetReconcileVersionPayload, ServErr,
};
use shared_internals::ClientCommand;

use crate::complex_state::router_handle;

pub async fn fetch_page_fingerprints() -> Result<GetPageFingerprintsPayload, ServErr> {
    let bytes = router_handle::send(ClientCommand::GetPageFingerprints).await?;
    decode_reply::<GetPageFingerprintsPayload>(&bytes)
}

pub async fn fetch_pages_by_ids(page_ids: Vec<String>) -> Result<GetAllYrsPagesPayload, ServErr> {
    let bytes = router_handle::send(ClientCommand::GetPagesByIds(page_ids)).await?;
    decode_reply::<GetAllYrsPagesPayload>(&bytes)
}

pub async fn fetch_reconcile_version() -> Result<GetReconcileVersionPayload, ServErr> {
    let bytes = router_handle::send(ClientCommand::GetReconcileVersion).await?;
    decode_reply::<GetReconcileVersionPayload>(&bytes)
}

pub async fn send_reconcile_letters(
    target_drift_version: String,
    target_reconcile_version: String,
    letters: Vec<Vec<u8>>,
) -> Result<(), ServErr> {
    let bytes = router_handle::send(ClientCommand::ReconcileLetters {
        target_drift_version,
        target_reconcile_version,
        letters,
    })
    .await?;
    decode_reply::<()>(&bytes)
}

/// Decode a `Result<T, ServErr>` bincode blob into its inner Result.
/// A decode failure surfaces as `ServErr::BadCode`.
fn decode_reply<T: serde::de::DeserializeOwned + serde::Serialize>(bytes: &[u8]) -> Result<T, ServErr> {
    let outer: Result<Result<T, ServErr>, protocol::error::DbError> =
        Convert::un_payloadify(bytes);
    match outer {
        Ok(inner) => inner,
        Err(e) => Err(ServErr::BadCode(format!("decode reply: {e:?}"))),
    }
}
