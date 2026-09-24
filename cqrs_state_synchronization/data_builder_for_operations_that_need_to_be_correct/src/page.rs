use error_stuff::cqrs_err::CqrsErr;
use protocol::serialization::Convert;
use std::sync::Arc;

use my_yrs_lib::{
    BossOfYrs,
    yrs_wrapper::{self, PositionToInsert},
};

use crate::{
    EverythingToInsertForNewPage,
    insert_structs::{
        BlocksToInsertCtx, NormalBlock, PagesInsertCtx, TitleBlock, UncommitedDiffsInsertCtx,
    },
};

use love_letter::LoveLetterSketch;

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
        yrs_wrapper::create_bookmark_of_synced_state(Arc::clone(&boss_of_yrs))?;

    let page_id = boss_of_yrs.page_id();
    let insert_page_ctx = PagesInsertCtx {
        page_id: page_id.clone(),
        blobbed_page: snapshot_of_yrs_doc.clone(),
        is_main_menu_page,
        version: yrs_representation_of_version_status,
    };
    // ---
    //insert_to_pages_tbl()
    // ---

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
    // insert_to_every_block_tbl()
    // ---

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
    // insert_to_uncommitted_diffs()
    // ---

    Ok(EverythingToInsertForNewPage {
        page_to_insert: insert_page_ctx,
        blocks_to_insert: blocks_to_insert_ctx,
        uncommitted_diffs: uncommitted_diffs_ctx,
    })
}

//
// ~~~
// HELPERS
// BELOW
// ~~~
//

/*
* struct PagesInsertCtx {
    page_id: String,
    blobbed_page: Vec<u8>,
    page_status: Vec<u8>,
    version: Vec<u8>,
    is_main_menu_page: bool,
}
*/

fn insert_title_and_empty_block(boss_of_yrs: Arc<BossOfYrs>) -> Result<IdOfTwoBlocks, CqrsErr> {
    /*
     * pub fn insert_new_block(self: Arc<Self>, block_content: String, block_meta_data: String, position: PositionToInsert) -> Result<String, YrsError>
     */
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

struct IdOfTwoBlocks {
    title_block_id: String,
    first_normal_block_id: String,
}

const FAKE_TIME: &str = "";

fn create_boss() -> Arc<BossOfYrs> {
    Arc::new(BossOfYrs::new("1".to_string(), FAKE_TIME.into()))
}
