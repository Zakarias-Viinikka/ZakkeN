use yrs::updates::decoder::Decode;
use yrs::{Doc, In, Map, ReadTxn, StateVector, Transact, Update};

pub struct YrsActivePages {
    doc: Doc,
}

const ACTIVE_PAGES_KEY: &str = "active_pages";

impl YrsActivePages {
    pub fn new(loaded_from_db: Vec<u8>) -> Self {
        let doc = Doc::new();
        if !loaded_from_db.is_empty() {
            let update =
                Update::decode_v1(&loaded_from_db).expect("Failed to decode active pages snapshot");
            doc.transact_mut()
                .apply_update(update)
                .expect("Failed to apply active pages snapshot");
        }
        Self { doc }
    }

    pub fn new_empty() -> Self {
        Self { doc: Doc::new() }
    }

    /// Mark a page as active (inserts or updates the key to `true`).
    pub fn mark_page_active(&mut self, page_id: &str) {
        let map = self.doc.get_or_insert_map(ACTIVE_PAGES_KEY);
        let mut txn = self.doc.transact_mut();
        map.insert(&mut txn, page_id.to_string(), In::from(true));
    }

    /// Mark a page as deleted (inserts or updates the key to `false`).
    pub fn mark_page_deleted(&mut self, page_id: &str) {
        let map = self.doc.get_or_insert_map(ACTIVE_PAGES_KEY);
        let mut txn = self.doc.transact_mut();
        map.insert(&mut txn, page_id.to_string(), In::from(false));
    }

    /// Returns `true` if the page is active, `false` if deleted.
    /// If the page is not in the map, returns `true` (assumed active by default).
    pub fn is_page_active(&self, page_id: &str) -> bool {
        let map = self.doc.get_or_insert_map(ACTIVE_PAGES_KEY);
        let txn = self.doc.transact();
        match map.get(&txn, page_id) {
            Some(value) => value.cast::<bool>().unwrap_or(true),
            None => true,
        }
    }

    pub fn snapshot(&self) -> Vec<u8> {
        self.doc.transact().encode_diff_v1(&StateVector::default())
    }

    pub fn merge_with_snapshot(&mut self, snapshot: &Vec<u8>) -> Result<(), String> {
        let update = Update::decode_v1(snapshot)
            .map_err(|e| format!("merge_with_snapshot: failed to decode update: {e}"))?;
        self.doc
            .transact_mut()
            .apply_update(update)
            .map_err(|e| format!("merge_with_snapshot: failed to apply update: {e}"))?;
        Ok(())
    }
}
