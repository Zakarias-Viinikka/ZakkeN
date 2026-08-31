#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused)]

use crate::yrs_wrapper;

use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use yrs::block::Item;
use yrs::types::ToJson;
use yrs::types::TypeRef::XmlText;
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use yrs::{
    Any, Array, ArrayRef, Doc, GetString, In, Map, MapPrelim, MapRef, ReadTxn, Text, TextRef,
    Transact, Update, XmlElementPrelim, XmlElementRef, XmlTextPrelim, XmlTextRef,
};
use yrs::{StateVector, Transaction};

use rand::prelude::*;

pub struct BossOfYrs {
    pub doc: Doc,
}

const BLOCKS_KEY: &str = "blocks";
const CONTENT_KEY: &str = "text";
const META_KEY: &str = "meta";
const ID_KEY: &str = "id";

impl BossOfYrs {
    pub fn new() -> Self {
        Self { doc: Doc::new() }
    }

    pub fn insert_new_block(&mut self, block_content: String, block_meta_data: String) {
        let block_id = self.generate_key() + "_clientid" + &self.doc.client_id().to_string();

        let text_as_xml_text_ref = XmlTextPrelim::new(block_content);
        let yrs_array_ref = self.doc.get_or_insert_array(BLOCKS_KEY.to_string());
        let mut txn = self.doc.transact_mut();

        let map = yrs_array_ref.push_back(
            &mut txn,
            MapPrelim::from([
                (ID_KEY.to_string(), In::from(block_id)),
                (CONTENT_KEY.to_string(), In::from(text_as_xml_text_ref)),
                (META_KEY.to_string(), In::from(block_meta_data)),
            ]),
        );
    }

    pub fn get_entire_page(&mut self) -> Result<Vec<Block>, String> {
        let array = self.doc.get_or_insert_array(BLOCKS_KEY.to_string());
        let txn = self.doc.transact();

        if array.len(&txn) == 0 {
            return Err(format!("no page found for id: {BLOCKS_KEY}"));
        }

        let page = array.iter(&txn);

        {
            let mut result = Vec::new();
            for block in page {
                if let Ok(block_map) = block.cast::<MapRef>() {
                    let text = block_map
                        .get(&txn, &CONTENT_KEY)
                        .and_then(|v| v.cast::<XmlTextRef>().ok())
                        .map(|t| t.get_string(&txn))
                        .ok_or_else(|| {
                            format!("read_page: failed to cast content field to XmlTextRef")
                        })?;
                    let meta = block_map
                        .get(&txn, &META_KEY)
                        .and_then(|v| v.cast::<String>().ok())
                        .ok_or_else(|| format!("read_page: failed to cast meta field to String"))?;
                    let id_in_yrs = block_map
                        .get(&txn, &ID_KEY)
                        .and_then(|v| v.cast::<String>().ok())
                        .ok_or_else(|| format!("read_page: failed to cast id field to String"))?;

                    result.push(Block {
                        text: text,
                        metadata: meta,
                        id_in_yrs: id_in_yrs,
                    });
                }
            }
            Ok(result)
        }
    }

    pub fn show_doc_info(&self) {
        let array = self.doc.get_or_insert_array(BLOCKS_KEY.to_string());
        let mut txn = self.doc.transact();

        let json_representation = array.to_json(&txn);
        println!("data blocks represented as json:");
        println!("{}", json_representation);
    }

    pub fn edit_text_block_insert(
        &mut self,
        block_id: &str,
        text_edit: TextEdit,
        edit_target: EditTarget,
    ) -> Result<(), String> {
        let array = self.doc.get_or_insert_array(BLOCKS_KEY.to_string());

        let block = block_from_block_id(self, block_id, array);
        let block = block.ok_or(format!("found no block with id: {block_id}"))?;

        self.edit_block(block, text_edit, edit_target)
    }

    fn edit_block(
        &mut self,
        block: MapRef,
        edit: TextEdit,
        edit_target: EditTarget,
    ) -> Result<(), String> {
        let mut txn = self.doc.transact_mut();

        match edit_target {
            EditTarget::Text => {
                let text_ref = block
                    .get(&txn, &CONTENT_KEY)
                    .ok_or_else(|| format!("edit_block: no text field"))?
                    .cast::<XmlTextRef>()
                    .map_err(|_| format!("edit_block: failed to cast text field to XmlTextRef"))?;

                match edit {
                    TextEdit::Insert(text, position) => {
                        text_ref.insert(&mut txn, u32_from_usize(position)?, &text)
                    }
                    TextEdit::Delete(text, position) => text_ref.remove_range(
                        &mut txn,
                        u32_from_usize(position)?,
                        u32_from_usize(text.chars().count())?,
                    ),
                    TextEdit::Replace {
                        old_text,
                        new_text,
                        position,
                    } => {
                        text_ref.remove_range(
                            &mut txn,
                            u32_from_usize(position)?,
                            u32_from_usize(old_text.chars().count())?,
                        );
                        text_ref.insert(&mut txn, u32_from_usize(position)?, &new_text)
                    }
                }

                Ok(())
            }

            EditTarget::Meta => {
                let current_meta = block
                    .get(&txn, &META_KEY)
                    .ok_or_else(|| "edit_block: no meta field".to_string())?
                    .cast::<String>()
                    .map_err(|_| "edit_block: failed to cast meta field to String".to_string())?;

                let new_meta = Self::apply_edit_to_string(current_meta, edit)?;

                block.insert(&mut txn, META_KEY.to_string(), In::from(new_meta));

                Ok(())
            }
        }
    }

    fn apply_edit_to_string(mut old_meta: String, edit: TextEdit) -> Result<String, String> {
        let mut chars: Vec<char> = old_meta.chars().collect();

        match edit {
            TextEdit::Insert(text, position) => {
                if position > chars.len() {
                    return Err("insert position exceeds string length".to_string());
                }

                chars.splice(position..position, text.chars());
            }

            TextEdit::Delete(text, position) => {
                let delete_len = text.chars().count();

                if position + delete_len > chars.len() {
                    return Err("delete range exceeds string length".to_string());
                }

                chars.drain(position..(position + delete_len));
            }

            TextEdit::Replace {
                old_text,
                new_text,
                position,
            } => {
                let old_len = old_text.chars().count();

                if position + old_len > chars.len() {
                    return Err("replace range exceeds string length".to_string());
                }

                chars.splice(position..(position + old_len), new_text.chars());
            }
        }

        Ok(chars.into_iter().collect())
    }

    pub fn read_block(&self, page_id: usize, block_id: &str) -> Option<String> {
        todo!()
    }

    pub fn generate_key(&self) -> String {
        let mut rng = rand::rng();
        let rnd_something: u64 = rng.random();
        return rnd_something.to_string() + "userid" + &self.get_user_id().to_string();

        //todo. just a temp gen for now
        /*
        i need to create a "generate new key method", but now i have the problem of 2 offline users working on the same page, having the small mathematical probability of generating the same key for 2 different blocks
        */
    }

    pub fn get_user_id(&self) -> u64 {
        self.doc.client_id().get()
    }

    pub fn snapshot(&self) -> Vec<u8> {
        let snapshot = self.doc.transact().encode_diff_v1(&StateVector::default());
        snapshot
    }

    pub fn merge_with(&mut self, other: &BossOfYrs) -> Result<(), String> {
        let sv = self.doc.transact().state_vector();
        let diff = other.doc.transact().encode_diff_v1(&sv);
        let update = Update::decode_v1(&diff)
            .map_err(|e| format!("merge_with: failed to decode update: {e}"))?;
        self.doc
            .transact_mut()
            .apply_update(update)
            .map_err(|e| format!("merge_with: failed to apply update: {e}"))?;
        Ok(())
    }

    pub fn merge_with_snapshot(&mut self, snapshot: &Vec<u8>) -> Result<(), String> {
        let update = Update::decode_v1(snapshot)
            .map_err(|e| format!("merge_with_snapshot: failed to decode update: {e}"))?;
        self.doc
            .transact_mut()
            .apply_update(update)
            .map_err(|e| format!("merge_with_snapshot: failed to apply update: {e}"))?;
        Ok(())
    }
}

pub fn doc_from_snapshot(snapshot: &Vec<u8>) -> Result<BossOfYrs, String> {
    let doc = yrs::Doc::new();
    let update = Update::decode_v1(snapshot)
        .map_err(|e| format!("doc_from_snapshot: failed to decode update: {e}"))?;
    doc.transact_mut()
        .apply_update(update)
        .map_err(|e| format!("doc_from_snapshot: failed to apply update: {e}"))?;
    let mut new_boss = BossOfYrs::new();
    new_boss.doc = doc;
    Ok(new_boss)
}

#[derive(Debug, PartialEq)]
pub enum TextEdit {
    Insert(String, usize),
    Delete(String, usize),
    Replace {
        old_text: String,
        new_text: String,
        position: usize,
    },
}

#[derive(Debug, PartialEq)]
pub enum EditTarget {
    Text,
    Meta,
}

pub struct Block {
    pub text: String,
    pub metadata: String,
    pub id_in_yrs: String,
}

fn block_from_block_id(boss: &BossOfYrs, block_id: &str, array_ref: ArrayRef) -> Option<MapRef> {
    let mut txn = boss.doc.transact();

    for block in array_ref.iter(&txn) {
        if let Ok(block_map) = block.cast::<MapRef>() {
            let id = block_map
                .get(&txn, ID_KEY)
                .and_then(|v| v.cast::<String>().ok());
            if id.as_deref() == Some(block_id) {
                return Some(block_map);
            }
        }
    }
    None
}

fn u32_from_usize(position: usize) -> Result<u32, String> {
    u32::try_from(position).map_err(|_| "position exceeds u32::MAX".to_string())
}
