#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused)]

use crate::yrs_wrapper::{self, BossOfYrs, TextEdit};
use std::collections::HashMap;
use yrs::block::Item;
use yrs::types::ToJson;
use yrs::types::TypeRef::XmlText;
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use yrs::{
    Any, Array, ArrayRef, Doc, GetString, Map, MapPrelim, MapRef, ReadTxn, Text, TextPrelim,
    Transact, Update, XmlElementPrelim, XmlElementRef, XmlTextPrelim, XmlTextRef,
};

/*
pub fn edit_insert(
    text_to_insert: &str,
    position_to_insert_at: usize,
    boss_of_yrs: &mut BossOfYrs,
    page_id: usize,
    text_block_id: &str,
) -> Result<(), String> {
    let text_block = boss_of_yrs
        .block_lookup
        .get(page_id)
        .and_then(|page| page.get(text_block_id))
        .ok_or_else(|| format!("block {} not found", text_block_id))?;

    let mut txn = boss_of_yrs.doc.transact_mut();
    text_block.insert(&mut txn, position_to_insert_at as u32, text_to_insert);
    Ok(())
}

pub fn edit_delete(
    position_to_delete_at: usize,
    length: usize,
    boss_of_yrs: &mut BossOfYrs,
    page_id: usize,
    text_block_id: &str,
) -> Result<(), String> {
    let text_block = boss_of_yrs
        .block_lookup
        .get(page_id)
        .and_then(|page| page.get(text_block_id))
        .ok_or_else(|| format!("block {} not found", text_block_id))?;
    let mut txn = boss_of_yrs.doc.transact_mut();
    text_block.remove_range(&mut txn, position_to_delete_at as u32, length as u32);
    Ok(())
}

pub fn edit_replace(
    text_to_insert: &str,
    position_to_delete_at: usize,
    length: usize,
    boss_of_yrs: &mut BossOfYrs,
    page_id: usize,
    text_block_id: &str,
) -> Result<(), String> {
    edit_delete(
        position_to_delete_at,
        length,
        boss_of_yrs,
        page_id,
        text_block_id,
    )?;
    edit_insert(
        text_to_insert,
        position_to_delete_at,
        boss_of_yrs,
        page_id,
        text_block_id,
    )?;
    Ok(())
}
*/
