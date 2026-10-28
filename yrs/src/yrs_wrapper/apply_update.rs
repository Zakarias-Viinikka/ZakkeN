use yrs::updates::decoder::Decode;
use yrs::{Doc, Transact, Update};

use super::errors::internal_error;
use crate::yrs_error::YrsError;

pub(crate) fn apply_update_bytes(
    doc: &Doc,
    bytes: &[u8],
    method: &str,
) -> Result<(), YrsError> {
    let update = Update::decode_v1(bytes)
        .map_err(|e| internal_error(format!("{method}: failed to decode update: {e}"), method))?;
    doc.transact_mut()
        .apply_update(update)
        .map_err(|e| internal_error(format!("{method}: failed to apply update: {e}"), method))?;
    Ok(())
}
