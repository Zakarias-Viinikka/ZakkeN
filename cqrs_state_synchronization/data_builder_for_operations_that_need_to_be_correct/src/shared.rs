pub struct PagesUpdateCtx {
    pub page_id: String,
    pub yrs_blob: Vec<u8>,
}

pub struct UncommitedDiffsInsertCtx {
    pub snapshot_of_edit: Vec<u8>,
    pub love_letter_sketch: Vec<u8>,
    pub session_id: String,
    pub target_id: String,
}
