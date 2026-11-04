uniffi::setup_scaffolding!();

pub mod anti_deadlock;
pub mod yrs_active_pages;
pub mod yrs_backlinks;
pub mod yrs_error;
pub mod yrs_wrapper;

pub use crate::anti_deadlock::*;
pub use crate::yrs_active_pages::YrsActivePages;
pub use crate::yrs_backlinks::YrsBacklinks;
pub use crate::yrs_error::YrsError;
pub use crate::yrs_wrapper::{
    Block, BossOfYrs, EditTarget, FreezeDocInput, FrozenBlock, TextEdit, doc_from_snapshot,
    freeze_doc, freeze_doc_builder,
};
