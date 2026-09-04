use yrs::updates::decoder::Decode;
use yrs::{Doc, In, Map, MapRef, ReadTxn, StateVector, Transact, Update};

pub struct YrsBacklinks {
    doc: Doc,
}

const BACKLINKS_KEY: &str = "backlinks";

impl YrsBacklinks {
    pub fn new(loaded_from_db: Vec<u8>) -> Self {
        let doc = Doc::new();
        if !loaded_from_db.is_empty() {
            let update =
                Update::decode_v1(&loaded_from_db).expect("Failed to decode backlinks snapshot");
            doc.transact_mut()
                .apply_update(update)
                .expect("Failed to apply backlinks snapshot");
        }
        Self { doc }
    }

    pub fn new_empty() -> Self {
        Self { doc: Doc::new() }
    }

    /// Add a backlink: `owner_of_backlink_id` (the page containing the link) points to `page_im_linking_to_id` (the target page).
    pub fn add_backlink(&mut self, owner_of_backlink_id: &str, page_im_linking_to_id: &str) {
        let backlinks_map = self.doc.get_or_insert_map(BACKLINKS_KEY);
        let mut txn = self.doc.transact_mut();

        // Get or create the inner map for this target page
        let inner_map: MapRef = backlinks_map.get_or_init(&mut txn, page_im_linking_to_id);

        // Insert the source page ID as key with value true (overwrite semantics)
        inner_map.insert(&mut txn, owner_of_backlink_id.to_string(), In::from(true));
    }

    /// Remove a backlink: `owner_of_backlink_id` no longer links to `page_im_linking_to_id`.
    /// Call this only when the source page no longer has any links to the target.
    pub fn remove_backlink(&mut self, owner_of_backlink_id: &str, page_im_linking_to_id: &str) {
        let backlinks_map = self.doc.get_or_insert_map(BACKLINKS_KEY);
        let mut txn = self.doc.transact_mut();

        if let Some(inner_map_ref) = backlinks_map.get(&txn, page_im_linking_to_id) {
            if let Ok(inner_map) = inner_map_ref.cast::<MapRef>() {
                inner_map.remove(&mut txn, owner_of_backlink_id);

                // Clean up empty inner map
                if inner_map.len(&txn) == 0 {
                    backlinks_map.remove(&mut txn, page_im_linking_to_id);
                }
            }
        }
    }

    /// Return all page IDs (owners) that link to `page_im_linking_to_id`.
    pub fn get_backlinks_for_page(&self, page_id: &str) -> Vec<String> {
        let backlinks_map = self.doc.get_or_insert_map(BACKLINKS_KEY);
        let txn = self.doc.transact();
        let mut result = Vec::new();

        if let Some(inner_map_ref) = backlinks_map.get(&txn, page_id) {
            if let Ok(inner_map) = inner_map_ref.cast::<MapRef>() {
                for (key, _) in inner_map.iter(&txn) {
                    result.push(key.to_string());
                }
            }
        }

        result
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
