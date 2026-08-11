#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(warnings)]
#[path = "../boss_of_yrs.rs"]
mod boss_of_yrs;
use boss_of_yrs::BossOfYrs;
use serde_json::json;
use std::sync::Arc;
use yrs::StateVector;
use yrs::block::Item;
use yrs::types::ToJson;
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use yrs::{Array, Doc, GetString, MapPrelim, ReadTxn, Text, TextPrelim, Transact, Update};

/*
 * using this as a reference for figuring out this yrs stuff
 * https://docs.rs/yrs/latest/yrs/#quick-start
 */

fn main() {
    let boss_of_yrs = BossOfYrs::new();
    let doc = &boss_of_yrs.doc;

    let remote_doc = BossOfYrs::new();

    let text_blocks = boss_of_yrs.doc.get_or_insert_array("text_blocks");

    {
        {
            let mut txn = boss_of_yrs.doc.transact_mut();
            let block = MapPrelim::from([
                ("Cat", "MainTitle"),
                ("text text text", "NormalText"),
                ("basd sdadas dsadas", "NormalText"),
            ]);

            let arrayLen = text_blocks.len(&txn);
            text_blocks.insert(&mut txn, arrayLen, block);
            //can use push back isntead. fix later
        }
        ///this was for updating and stuff i think?
        /*
                {
                    let mut remote_txn = remote_doc.doc.transact_mut();
                    //stores info about what changes the remote document has and turns into a format that is cheaper to transmit over the network for example.
                    let state_vector = remote_txn.state_vector().encode_v1();
                    let update = txn.encode_diff_v1(&StateVector::decode_v1(&state_vector).unwrap());
                    remote_txn.apply_update(Update::decode_v1(update.as_slice()).unwrap());
                }
        */
        {
            let mut txn = boss_of_yrs.doc.transact();
            let read_array_from_doc = txn.get_array("text_blocks");
            let read_array_from_doc = read_array_from_doc.unwrap();
            let read_array_from_doc = read_array_from_doc.iter(&txn);

            let mut tmp: String = "".to_string();
            for x in read_array_from_doc {
                let x = x.to_string(&txn);
                tmp.push_str(&x);
            }

            println!("{:?}", tmp);
            //println!("{}", text[0].get_string(&boss_of_yrs.doc.transact()));
        }
    }

    //i can't use transact on the remote txn in the same scope because the transact mut hostages it for some reason

    //println!("{}", text[0].get_string(&remote_doc.doc.transact()));
    //println!("{}", text[1].get_string(&remote_doc.doc.transact()));

    //println!("{}", text[0].get_string(&boss_of_yrs.doc.transact()));
    //println!("{}", text[1].get_string(&boss_of_yrs.doc.transact()));
    //println!("{}", serde_json::to_string_pretty(&map).unwrap());
}
