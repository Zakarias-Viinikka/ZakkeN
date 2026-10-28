use yrs::{Array, ArrayRef, GetString, Map, MapRef, ReadTxn, Text, XmlTextRef};

use super::keys::{CONTENT_KEY, FRAC_POSITION_KEY, ID_KEY, META_KEY};
use super::types::Block;
use crate::yrs_error::YrsError;

pub(crate) fn string_field<T: ReadTxn>(
    map: &MapRef,
    txn: &T,
    key: &str,
) -> Option<String> {
    map.get(txn, key).and_then(|v| v.cast::<String>().ok())
}

pub(crate) fn find_block<T: ReadTxn>(
    array: &ArrayRef,
    txn: &T,
    block_id: &str,
) -> Result<(u32, MapRef), YrsError> {
    for (index, block) in array.iter(txn).enumerate() {
        if let Ok(block_map) = block.cast::<MapRef>() {
            if string_field(&block_map, txn, ID_KEY).as_deref() == Some(block_id) {
                return Ok((index as u32, block_map));
            }
        }
    }
    Err(crate::generic_error!(
        format!("no block with id {block_id}"),
        "find_block_with_id"
    ))
}

pub(crate) fn get_block_map_at<T: ReadTxn>(
    array: &ArrayRef,
    txn: &T,
    position: usize,
) -> Result<MapRef, YrsError> {
    let block = array.iter(txn).nth(position).ok_or_else(|| {
        crate::generic_error!(
            format!("no block at position {position}"),
            "get_block_map_at"
        )
    })?;
    block.cast::<MapRef>().map_err(|_| {
        crate::generic_error!(
            format!("block at position {position} is not a map"),
            "get_block_map_at"
        )
    })
}

pub(crate) fn frac_position_at<T: ReadTxn>(
    array: &ArrayRef,
    txn: &T,
    position: usize,
) -> Result<String, YrsError> {
    let block_map = get_block_map_at(array, txn, position)?;
    string_field(&block_map, txn, FRAC_POSITION_KEY).ok_or_else(|| {
        crate::generic_error!(
            format!("block at position {position} missing {FRAC_POSITION_KEY}"),
            "get_frac_position_of_block_at"
        )
    })
}

pub(crate) fn frac_position_by_id<T: ReadTxn>(
    array: &ArrayRef,
    txn: &T,
    block_id: &str,
) -> Result<String, YrsError> {
    let (_, block_map) = find_block(array, txn, block_id)?;
    string_field(&block_map, txn, FRAC_POSITION_KEY).ok_or_else(|| {
        crate::generic_error!(
            format!("block {block_id} missing {FRAC_POSITION_KEY}"),
            "get_frac_position_by_id"
        )
    })
}

pub(crate) fn read_block_at<T: ReadTxn>(
    array: &ArrayRef,
    txn: &T,
    position: usize,
) -> Result<(String, Block), YrsError> {
    let block_map = get_block_map_at(array, txn, position)?;
    let text = block_map
        .get(txn, CONTENT_KEY)
        .and_then(|v| v.cast::<XmlTextRef>().ok())
        .map(|t| t.get_string(txn))
        .ok_or_else(|| {
            crate::generic_error!(
                format!("block at position {position} content is not XmlTextRef"),
                "read_block_at"
            )
        })?;
    let metadata = string_field(&block_map, txn, META_KEY).ok_or_else(|| {
        crate::generic_error!(
            format!("block at position {position} meta is not a string"),
            "read_block_at"
        )
    })?;
    let id_in_yrs = string_field(&block_map, txn, ID_KEY).ok_or_else(|| {
        crate::generic_error!(
            format!("block at position {position} id is not a string"),
            "read_block_at"
        )
    })?;
    let frac_position = string_field(&block_map, txn, FRAC_POSITION_KEY).ok_or_else(|| {
        crate::generic_error!(
            format!("block at position {position} missing {FRAC_POSITION_KEY}"),
            "read_block_at"
        )
    })?;
    Ok((frac_position, Block { text, metadata, id_in_yrs }))
}
