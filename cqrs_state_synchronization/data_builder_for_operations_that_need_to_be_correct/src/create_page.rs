use error_stuff::cqrs_err::CqrsErr;
use love_letter::LoveLetterSketch;
use my_yrs_lib::{
    BossOfYrs,
    yrs_wrapper::{self, PositionToInsert},
};
use protocol::serialization::Convert;
use std::sync::Arc;

use crate::shared::UncommitedDiffsInsertCtx;

pub struct EverythingToInsertForNewPage {
    pub page_to_insert: PagesInsertCtx,
    pub blocks_to_insert: BlocksToInsertCtx,
    pub uncommitted_diffs: UncommitedDiffsInsertCtx,
}

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
    pub page_id: String,
    pub blobbed_page: Vec<u8>,
    pub version: Vec<u8>,
    pub is_main_menu_page: bool,
}

pub fn create_page(
    is_main_menu_page: bool,
    session_id: String,
) -> Result<EverythingToInsertForNewPage, CqrsErr> {
    // ---
    //insert_to_pages_tbl()
    // ---
    let boss_of_yrs = create_boss();
    let two_ids = insert_title_and_empty_block(Arc::clone(&boss_of_yrs))?;
    let snapshot_of_yrs_doc = BossOfYrs::snapshot(Arc::clone(&boss_of_yrs))?;

    let yrs_representation_of_version_status =
        yrs_wrapper::create_bookmark(Arc::clone(&boss_of_yrs))?;

    let page_id = boss_of_yrs.page_id();
    let insert_page_ctx = PagesInsertCtx {
        page_id: page_id.clone(),
        blobbed_page: snapshot_of_yrs_doc.clone(),
        is_main_menu_page,
        version: yrs_representation_of_version_status,
    };

    // ---
    // insert_to_every_block_tbl()
    // ---

    let is_part_of_main_menu_page = is_main_menu_page;
    let title_block = TitleBlock {
        my_id_as_given_by_yrs: two_ids.title_block_id,
        content: "".to_string(),
        id_of_page_i_belong_to: page_id.clone(),
        position: 0.0,
        is_part_of_main_menu_page,
    };

    let normal_block = NormalBlock {
        my_id_as_given_by_yrs: two_ids.first_normal_block_id,
        content: "".to_string(),
        id_of_page_i_belong_to: page_id.clone(),
        position: 1.0,
        is_part_of_main_menu_page,
    };

    let blocks_to_insert_ctx = BlocksToInsertCtx {
        title_block,
        normal_block,
    };

    // ---
    // insert_to_uncommitted_diffs()
    // ---

    let snapshot_of_edit = snapshot_of_yrs_doc;
    let love_letter_sketch = LoveLetterSketch::CreateNewPage {
        page_id: page_id.clone(),
    }
    .to_payload();

    let uncommitted_diffs_ctx = UncommitedDiffsInsertCtx {
        snapshot_of_edit,
        love_letter_sketch,
        session_id,
        target_id: page_id,
    };
    // ---

    Ok(EverythingToInsertForNewPage {
        page_to_insert: insert_page_ctx,
        blocks_to_insert: blocks_to_insert_ctx,
        uncommitted_diffs: uncommitted_diffs_ctx,
    })
}

struct IdOfTwoBlocks {
    title_block_id: String,
    first_normal_block_id: String,
}

const FAKE_TIME: &str = "";

fn create_boss() -> Arc<BossOfYrs> {
    Arc::new(BossOfYrs::new("1".to_string(), FAKE_TIME.into()))
}

fn insert_title_and_empty_block(boss_of_yrs: Arc<BossOfYrs>) -> Result<IdOfTwoBlocks, CqrsErr> {
    let id1 = BossOfYrs::insert_new_block(
        Arc::clone(&boss_of_yrs),
        "".to_string(),
        "".to_string(),
        PositionToInsert::AtEnd,
    )?; //title
    let id2 = BossOfYrs::insert_new_block(
        boss_of_yrs,
        "".to_string(),
        "".to_string(),
        PositionToInsert::AtEnd,
    )?; //empty block to be written in.

    Ok(IdOfTwoBlocks {
        title_block_id: id1,
        first_normal_block_id: id2,
    })
}
