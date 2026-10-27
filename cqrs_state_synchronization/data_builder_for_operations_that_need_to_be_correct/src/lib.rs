uniffi::setup_scaffolding!();

pub mod add_block;
pub mod create_page;
pub mod delete_block;
pub mod diff_result_to_text_edit_conversion;
pub mod disable_page;
pub mod edit_block;
pub mod move_block;
pub mod shared;
pub mod uniffi_exports;

pub use add_block::{EverythingForAddBlock, EveryBlockInExistenceInsertCtx, add_block};
pub use create_page::{
    BlocksToInsertCtx, EverythingToInsertForNewPage, NormalBlock, PagesInsertCtx, TitleBlock,
    create_page,
};
pub use delete_block::{EverythingForDeleteBlock, delete_block};
pub use disable_page::{EverythingForDisablePage, disable_page};
pub use edit_block::{EveryBlockInExistenceUpdateCtx, EverythingForEditBlock, edit_block};
pub use move_block::{EverythingForMoveBlock, move_block};
pub use shared::{ClientLoveLetterInsertCtx, PagesUpdateCtx};

pub mod merge_page;
pub use merge_page::{
    build_merge_blob, build_merge_disable, build_merge_new_page, ExistingBlock, FinalBlock,
    MergePlan, NewPageInfo, ReconcileOp,
};
