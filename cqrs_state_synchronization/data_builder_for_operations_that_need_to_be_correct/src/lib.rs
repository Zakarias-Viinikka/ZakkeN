uniffi::setup_scaffolding!();

pub mod add_block;
pub mod create_page;
pub mod delete_block;
pub mod diff_result_to_text_edit_conversion;
pub mod disable_page;
pub mod edit_block;
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
pub use shared::{ClientLoveLetterInsertCtx, PagesUpdateCtx};