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
    Any, Array, ArrayRef, Doc, GetString, Map, MapPrelim, ReadTxn, Text, TextRef, Transact, Update,
    XmlElementPrelim, XmlElementRef, XmlTextPrelim, XmlTextRef,
};
use yrs::{StateVector, Transaction};

use rand::prelude::*;

pub struct BossOfYrs {
    pub doc: Doc,
    pub block_lookup: Vec<HashMap<String, XmlTextRef>>, //the Hashmap is a lookup for "i have this text_block_id how do i quickly get the right TextRef"
}

impl BossOfYrs {
    pub fn new() -> Self {
        Self {
            doc: Doc::new(),
            block_lookup: Vec::new(),
        }
    }

    pub fn insert_new_block(
        &self,
        block_content: String,
        block_meta_data: String,
        block_id: String,
    ) {
        {
            let array_length = {
                let tmp_data_blocks = self.doc.get_or_insert_array(doc_block_id());
                let mut txn = self.doc.transact();
                tmp_data_blocks.len(&txn)
            };
            let block_content = XmlTextPrelim::new(format!("{{{}}} {}", block_id, block_content));
            //let block_meta_data = XmlElementPrelim::new(block_meta_data, []);

            // IMPORTANT
            // for some reason, if i put this code after
            // ...txn =...
            // then my code deadlocks.
            let data_blocks = self.doc.get_or_insert_array(doc_block_id()); //"text_blocks");

            let mut txn = self.doc.transact_mut();

            data_blocks.insert(&mut txn, array_length, block_content);
        }
        /*
        let data_blocks_metadata = self.doc.get_or_insert_map("text_blocks_metadata");
        data_blocks_metadata.insert(&mut txn, block_id, block_meta_data);
        */
    }

    pub fn show_doc_info(&self) {
        let array = self.doc.get_or_insert_array(doc_block_id());
        let mut txn = self.doc.transact();

        let json_representation = array.to_json(&txn);
        println!("data blocks represented as json:");
        println!("{}", json_representation);
        /*if let Some(mapRef) = txn {
            println!("{}", mapRef.to_json(self.doc.transact()));
            //dbg!(data);
        }*/
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

fn doc_block_id() -> String {
    "text_blocks".to_string()
}
