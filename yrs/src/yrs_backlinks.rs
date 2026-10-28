use std::sync::{Arc, RwLock};

use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use yrs::{Doc, Map, ReadTxn, StateVector, Transact, Update};

use crate::yrs_error::YrsError;
use crate::yrs_wrapper::apply_update_bytes;
use crate::yrs_wrapper::internal_error;

const STATE_MAP_KEY: &str = "state";
const DISABLED_KEY: &str = "disabled";

#[derive(uniffi::Object)]
pub struct YrsBacklinks {
    doc: RwLock<Doc>,
}

#[uniffi::export]
impl YrsBacklinks {
    #[uniffi::constructor]
    pub fn new_empty() -> Self {
        Self {
            doc: RwLock::new(Doc::new()),
        }
    }

    #[uniffi::constructor]
    pub fn new(loaded_from_db: Vec<u8>) -> Result<Self, YrsError> {
        let doc = Doc::new();
        if !loaded_from_db.is_empty() {
            let update = Update::decode_v1(&loaded_from_db).map_err(|e| {
                internal_error(
                    format!("Failed to decode backlinks snapshot: {e}"),
                    "new",
                )
            })?;
            doc.transact_mut().apply_update(update).map_err(|e| {
                internal_error(
                    format!("Failed to apply backlinks snapshot: {e}"),
                    "new",
                )
            })?;
        }
        Ok(Self {
            doc: RwLock::new(doc),
        })
    }

    pub fn set_disabled(self: Arc<Self>, disabled: bool) -> Result<(), YrsError> {
        crate::guarded!("set_disabled", move || {
            let doc = crate::write_doc!(self.doc, "set_disabled");
            let state_map = doc.get_or_insert_map(STATE_MAP_KEY);
            let mut txn = doc.transact_mut();
            state_map.insert(&mut txn, DISABLED_KEY, disabled);
            Ok(())
        })
    }

    pub fn is_disabled(self: Arc<Self>) -> Result<bool, YrsError> {
        crate::guarded!("is_disabled", move || {
            let doc = crate::read_doc!(self.doc, "is_disabled");
            let state_map = doc.get_or_insert_map(STATE_MAP_KEY);
            let txn = doc.transact();
            match state_map.get(&txn, DISABLED_KEY) {
                Some(value) => value.cast::<bool>().map_err(|_| {
                    crate::generic_error!(
                        "failed to cast disabled to bool",
                        "is_disabled"
                    )
                }),
                None => Ok(false),
            }
        })
    }

    pub fn snapshot(self: Arc<Self>) -> Result<Vec<u8>, YrsError> {
        crate::guarded!("snapshot", move || {
            let doc = crate::read_doc!(self.doc, "snapshot");
            Ok(doc.transact().encode_diff_v1(&StateVector::default()))
        })
    }

    pub fn merge_with_snapshot(self: Arc<Self>, snapshot: Vec<u8>) -> Result<(), YrsError> {
        crate::guarded!("merge_with_snapshot", move || {
            let doc = crate::write_doc!(self.doc, "merge_with_snapshot");
            apply_update_bytes(&doc, &snapshot, "merge_with_snapshot")
        })
    }

    pub fn create_bookmark(self: Arc<Self>) -> Result<Vec<u8>, YrsError> {
        crate::guarded!("create_bookmark", move || {
            let doc = crate::read_doc!(self.doc, "create_bookmark");
            Ok(doc.transact().state_vector().encode_v1())
        })
    }

    pub fn generate_diff_snapshot(
        self: Arc<Self>,
        bookmark_serialized: Vec<u8>,
    ) -> Result<Vec<u8>, YrsError> {
        crate::guarded!("generate_diff_snapshot", move || {
            let bookmark = StateVector::decode_v1(&bookmark_serialized).map_err(|e| {
                internal_error(
                    format!("generate_diff_snapshot: failed to decode bookmark: {e}"),
                    "generate_diff_snapshot",
                )
            })?;
            let doc = crate::read_doc!(self.doc, "generate_diff_snapshot");
            Ok(doc.transact().encode_diff_v1(&bookmark))
        })
    }
}