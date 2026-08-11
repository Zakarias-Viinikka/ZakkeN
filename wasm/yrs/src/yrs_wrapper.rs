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
    Any, Array, ArrayRef, Doc, GetString, In, Map, MapPrelim, ReadTxn, Text, TextRef, Transact,
    Update, XmlElementPrelim, XmlElementRef, XmlTextPrelim, XmlTextRef,
};
use yrs::{StateVector, Transaction};

use rand::prelude::*;

pub struct BossOfYrs {
    pub doc: Doc,
    content_key: String,
    meta_key: String,
}

impl BossOfYrs {
    pub fn new() -> Self {
        Self {
            doc: Doc::new(),
            content_key: "text".to_string(),
            meta_key: "meta".to_string(),
        }
    }

    pub fn insert_new_block(
        &mut self,
        block_content: String,
        block_meta_data: String,
        page_id: String,
    ) {
        let text_as_xml_text_ref = XmlTextPrelim::new(block_content);
        let yrs_array_ref = self.doc.get_or_insert_array(page_id.to_string());
        let mut txn = self.doc.transact_mut();

        let map = yrs_array_ref.push_back(
            &mut txn,
            MapPrelim::from([
                (self.content_key.clone(), In::from(text_as_xml_text_ref)),
                (self.meta_key.clone(), In::from(block_meta_data)),
            ]),
        );
    }

    pub fn get_entire_page(page_id: String) -> Result<Page, String> {
        todo!();
        Err(format!("no page found for id: {page_id}"))
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

    pub fn read_block(&self, page_id: usize, block_id: &str) -> Option<Page> {
        todo!()
    }

    pub fn do_change(&mut self, block_id: &str, change: TextEdit) {}

    pub fn update_metadata(&mut self, block_id: &str, change: TextEdit) {
        todo!()
    }
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

pub struct Page {
    blocks: Vec<Block>,
}

pub struct Block {
    pub text: String,
    pub metadata: String,
    pub id_in_yrs: String,
}

pub fn generate_key(user_id: &str) -> String {
    let mut rng = rand::rng();
    let rnd_something: u64 = rng.random();
    dbg!(rnd_something);
    return rnd_something.to_string() + "userid" + user_id;

    //todo. just a temp gen for now
    /*
    i need to create a "generate new key method", but now i have the problem of 2 offline users working on the same page, having the small mathematical probability of generating the same key for 2 different blocks
    */
}
