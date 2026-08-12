#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused)]

use crate::{helper, yrs_wrapper};

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
    content_key: String,
    meta_key: String,
    id_key: String,
}

impl BossOfYrs {
    pub fn new() -> Self {
        Self {
            doc: Doc::new(),
            content_key: "text".to_string(),
            meta_key: "meta".to_string(),
            id_key: "id".to_string(),
        }
    }

    pub fn insert_new_block(
        &mut self,
        block_content: String,
        block_meta_data: String,
        page_id: String,
        counter_for_block_id: &mut u32,
    ) {
        let block_id = self.generate_key() + "_counter:" + &counter_for_block_id.to_string();

        let text_as_xml_text_ref = XmlTextPrelim::new(block_content);
        let yrs_array_ref = self.doc.get_or_insert_array(page_id.to_string());
        let mut txn = self.doc.transact_mut();

        let map = yrs_array_ref.push_back(
            &mut txn,
            MapPrelim::from([
                (self.id_key.clone(), In::from(block_id)),
                (self.content_key.clone(), In::from(text_as_xml_text_ref)),
                (self.meta_key.clone(), In::from(block_meta_data)),
            ]),
        );
    }

    pub fn get_entire_page(&mut self, page_id: String) -> Result<Vec<Block>, String> {
        let array = self.doc.get_or_insert_array(page_id.clone());
        let txn = self.doc.transact();

        if array.len(&txn) == 0 {
            return Err(format!("no page found for id: {page_id}"));
        }

        let page = array.iter(&txn);

        {
            let mut result = Vec::new();
            for block in page {
                if let Ok(block_map) = block.cast::<MapRef>() {
                    let text = block_map
                        .get(&txn, &self.content_key)
                        .and_then(|v| v.cast::<XmlTextRef>().ok())
                        .map(|t| t.get_string(&txn))
                        .ok_or_else(|| {
                            format!("read_page: failed to cast content field to XmlTextRef")
                        })?;
                    let meta = block_map
                        .get(&txn, &self.meta_key)
                        .and_then(|v| v.cast::<String>().ok())
                        .ok_or_else(|| format!("read_page: failed to cast meta field to String"))?;
                    let id_in_yrs = block_map
                        .get(&txn, &self.id_key)
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

    pub fn show_doc_info(&self, page_id: String) {
        let array = self.doc.get_or_insert_array(page_id);
        let mut txn = self.doc.transact();

        let json_representation = array.to_json(&txn);
        println!("data blocks represented as json:");
        println!("{}", json_representation);
    }

    /*pub fn read_page(&self, page_id: usize) -> Vec<Page> {
        let txn = self.doc.transact();
        let page = match self.block_lookup.get(page_id) {
            Some(p) => p,
            None => return Vec::new(),
        };

        page.values()
            .map(|text_ref| Page {
                text: text_ref.get_string(&txn),
                metadata: String::new(),
            })
            .collect()
    }*/

    pub fn read_block(&self, page_id: usize, block_id: &str) -> Option<String> {
        todo!()
    }

    pub fn do_change(&mut self, block_id: &str, change: TextEdit) {}

    pub fn update_metadata(&mut self, block_id: &str, change: TextEdit) {
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

pub struct Block {
    pub text: String,
    pub metadata: String,
    pub id_in_yrs: String,
}
