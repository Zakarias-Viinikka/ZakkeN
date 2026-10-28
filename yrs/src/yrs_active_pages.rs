use std::sync::{Arc, RwLock};

use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use yrs::{Doc, In, Map, ReadTxn, StateVector, Transact, Update};

use crate::yrs_error::YrsError;
use crate::yrs_wrapper::apply_update_bytes;
use crate::yrs_wrapper::internal_error;

const ACTIVE_PAGES_KEY: &str = "active_pages";

#[derive(uniffi::Object)]
pub struct YrsActivePages {
    doc: RwLock<Doc>,
}

#[uniffi::export]
impl YrsActivePages {
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
                    format!("Failed to decode active pages snapshot: {e}"),
                    "new",
                )
            })?;
            doc.transact_mut().apply_update(update).map_err(|e| {
                internal_error(
                    format!("Failed to apply active pages snapshot: {e}"),
                    "new",
                )
            })?;
        }
        Ok(Self {
            doc: RwLock::new(doc),
        })
    }

    pub fn mark_page_active(self: Arc<Self>, page_id: String) -> Result<(), YrsError> {
        crate::guarded!("mark_page_active", move || {
            let doc = crate::write_doc!(self.doc, "mark_page_active");
            let map = doc.get_or_insert_map(ACTIVE_PAGES_KEY);
            let mut txn = doc.transact_mut();
            map.insert(&mut txn, page_id, In::from(true));
            Ok(())
        })
    }

    pub fn mark_page_deleted(self: Arc<Self>, page_id: String) -> Result<(), YrsError> {
        crate::guarded!("mark_page_deleted", move || {
            let doc = crate::write_doc!(self.doc, "mark_page_deleted");
            let map = doc.get_or_insert_map(ACTIVE_PAGES_KEY);
            let mut txn = doc.transact_mut();
            map.insert(&mut txn, page_id, In::from(false));
            Ok(())
        })
    }

    pub fn is_page_active(self: Arc<Self>, page_id: String) -> Result<bool, YrsError> {
        crate::guarded!("is_page_active", move || {
            let doc = crate::read_doc!(self.doc, "is_page_active");
            let map = doc.get_or_insert_map(ACTIVE_PAGES_KEY);
            let txn = doc.transact();
            Ok(match map.get(&txn, &page_id) {
                Some(value) => value.cast::<bool>().unwrap_or(true),
                None => true,
            })
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