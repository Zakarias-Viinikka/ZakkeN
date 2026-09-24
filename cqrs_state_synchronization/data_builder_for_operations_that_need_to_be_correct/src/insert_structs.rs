pub struct EverythingToInsertForNewPage {
    pub page_to_insert: PagesInsertCtx,
    pub blocks_to_insert: BlocksToInsertCtx,
    pub uncommitted_diffs: UncommitedDiffsInsertCtx,
}

pub struct BlocksToInsertCtx {
    pub title_id: String,
    pub first_normal_block_id: String,
}

pub struct PagesInsertCtx {
    pub page_id: String, //creating a new page asks for a user id, then yrs generates a unique id for that page based on the id
    pub table_name: String,
    pub blobbed_page: Vec<u8>,
    pub page_status: Vec<u8>,
    pub version: Vec<u8>,
    pub is_main_menu_page: bool,
}

pub struct UncommitedDiffsInsertCtx {
    pub snapshot_of_edit: Vec<u8>,
    pub love_letter_sketch: Vec<u8>,
    pub session_id: String,
    pub target_id: String,
}
