use std::sync::Arc;

use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use yrs::{Doc, ReadTxn, StateVector, Transact};

use super::apply_update::apply_update_bytes;
use super::boss::BossOfYrs;
use super::errors::internal_error;
use crate::yrs_error::YrsError;

#[uniffi::export]
pub fn doc_from_snapshot(
    snapshot: Vec<u8>,
    user_id: String,
    page_id: String,
    time: String,
) -> Result<BossOfYrs, YrsError> {
    crate::guarded!("doc_from_snapshot", move || {
        let doc = Doc::new();
        apply_update_bytes(&doc, &snapshot, "doc_from_snapshot")?;
        Ok(BossOfYrs {
            doc: std::sync::RwLock::new(doc),
            page_id,
            user_id,
            time,
        })
    })
}

#[uniffi::export]
pub fn create_bookmark(boss: Arc<BossOfYrs>) -> Result<Vec<u8>, YrsError> {
    crate::guarded!("create_bookmark", move || {
        let doc = crate::read_doc!(boss.doc, "create_bookmark");
        Ok(doc.transact().state_vector().encode_v1())
    })
}

#[uniffi::export]
pub fn generate_diff_snapshot(
    boss: Arc<BossOfYrs>,
    bookmark_serialized: Vec<u8>,
) -> Result<Vec<u8>, YrsError> {
    crate::guarded!("generate_diff_snapshot", move || {
        let bookmark = StateVector::decode_v1(&bookmark_serialized).map_err(|e| {
            internal_error(
                format!("deserialize_bookmark: failed to decode: {e}"),
                "deserialize_bookmark",
            )
        })?;
        let doc = crate::read_doc!(boss.doc, "generate_diff_snapshot");
        Ok(doc.transact().encode_diff_v1(&bookmark))
    })
}
