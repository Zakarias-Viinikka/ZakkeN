use smart_frac_index::frac_core as frac_core;
use yrs::{ArrayRef, ReadTxn};

use super::block_lookup::{frac_position_at, frac_position_by_id};
use crate::yrs_error::YrsError;

pub(crate) fn compute_new_frac_position_in<T: ReadTxn>(
    array: &ArrayRef,
    txn: &T,
    block_id_of_above: Option<&str>,
    block_id_of_below: Option<&str>,
) -> Result<String, YrsError> {
    match (block_id_of_above, block_id_of_below) {
        (Some(above), Some(below)) => {
            let above_frac = frac_position_by_id(array, txn, above)?;
            let below_frac = frac_position_by_id(array, txn, below)?;
            Ok(frac_core::key_between(&above_frac, &below_frac)?)
        }
        (Some(above), None) => {
            let above_frac = frac_position_by_id(array, txn, above)?;
            Ok(frac_core::key_after(&above_frac)?)
        }
        (None, Some(below)) => {
            let below_frac = frac_position_by_id(array, txn, below)?;
            Ok(frac_core::key_before(&below_frac)?)
        }
        (None, None) => Err(crate::generic_error!(
            "at least one neighbour is required",
            "compute_new_frac_position_in"
        )),
    }
}

pub(crate) fn get_new_frac_position<T: ReadTxn>(
    array: &ArrayRef,
    txn: &T,
    insert_at: u32,
    len: u32,
) -> Result<String, YrsError> {
    if len == 0 {
        return Ok(frac_core::key_at_start());
    }
    if insert_at == 0 {
        let first = frac_position_at(array, txn, 0)?;
        return Ok(frac_core::key_before(&first)?);
    }
    if insert_at == len {
        let last = frac_position_at(array, txn, (len - 1) as usize)?;
        return Ok(frac_core::key_after(&last)?);
    }
    let prev = frac_position_at(array, txn, (insert_at - 1) as usize)?;
    let next = frac_position_at(array, txn, insert_at as usize)?;
    Ok(frac_core::key_between(&prev, &next)?)
}
