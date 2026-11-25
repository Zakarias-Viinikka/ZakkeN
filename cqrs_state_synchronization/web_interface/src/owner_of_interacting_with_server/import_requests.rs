//! Caller pov: ask the server for one batch of pages from an import zip.
//! The caller owns the cursor: pass `None` the first time, then the last
//! page_id it received, until `has_more` is false.

use server_protocol::{ImportPagesPayload, ImportPagesReply, ServErr};
use shared_internals::ClientCommand;

use super::reconcile_requests::decode_reply;
use crate::complex_state::router_handle;

pub async fn fetch_import_pages(
    zip_path: String,
    after_page_id: Option<String>,
    limit: usize,
) -> Result<ImportPagesReply, ServErr> {
    let bytes = router_handle::send(ClientCommand::ImportPages(ImportPagesPayload {
        zip_path,
        after_page_id,
        limit,
    }))
    .await?;
    decode_reply::<ImportPagesReply>(&bytes)
}
