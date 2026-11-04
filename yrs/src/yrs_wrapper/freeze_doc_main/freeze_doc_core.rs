use crate::yrs_error::YrsError;
use crate::yrs_wrapper::BossOfYrs;

use super::freeze_internals::{freeze_doc_internals, FreezeDocInput};

#[uniffi::export]
pub fn freeze_doc(input: FreezeDocInput) -> Result<BossOfYrs, YrsError> {
    crate::guarded!("freeze_doc", move || freeze_doc_internals(input))
}
