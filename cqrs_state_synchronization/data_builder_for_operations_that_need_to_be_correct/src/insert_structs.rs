pub struct EverythingToInsertForNewPage {
    pub page_to_insert: PagesInsertCtx,
    pub blocks_to_insert: BlocksToInsertCtx,
    pub uncommitted_diffs: UncommitedDiffsInsertCtx,
}

pub struct EverythingForEditBlock {
    pub pages_update: PagesUpdateCtx,
    pub every_block_in_existence_update: EveryBlockInExistenceUpdateCtx,
    pub uncommitted_diffs: UncommitedDiffsInsertCtx,
}

pub struct EverythingForDisablePage {
    pub pages_update: PagesUpdateCtx,
    pub uncommitted_diffs: UncommitedDiffsInsertCtx,
}

pub struct PagesUpdateCtx {
    pub page_id: String,
    pub new_blobbed_page: Vec<u8>,
}

pub struct EveryBlockInExistenceUpdateCtx {
    pub block_id: String,
    pub new_content: String,
}
//
pub struct BlocksToInsertCtx {
    pub title_block: TitleBlock,
    pub normal_block: NormalBlock,
}

pub struct TitleBlock {
    pub my_id_as_given_by_yrs: String,
    pub content: String,
    pub id_of_page_i_belong_to: String,
    pub position: f64,
    pub is_part_of_main_menu_page: bool,
}

pub struct NormalBlock {
    pub my_id_as_given_by_yrs: String,
    pub content: String,
    pub id_of_page_i_belong_to: String,
    pub position: f64,
    pub is_part_of_main_menu_page: bool,
}

pub struct PagesInsertCtx {
    pub page_id: String, //creating a new page asks for a user id, then yrs generates a unique id for that page based on the id
    pub blobbed_page: Vec<u8>,
    pub version: Vec<u8>,
    pub is_main_menu_page: bool,
}

pub struct UncommitedDiffsInsertCtx {
    pub snapshot_of_edit: Vec<u8>,
    pub love_letter_sketch: Vec<u8>,
    pub session_id: String,
    pub target_id: String,
}
