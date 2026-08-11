#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(warnings)]
#[path = "../boss_of_yrs.rs"]
mod boss_of_yrs;
use boss_of_yrs::BossOfYrs;
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use yrs::{Doc, GetString, ReadTxn, Text, Transact, Update};

/*
 * using this as a reference for figuring out this yrs stuff
 * https://docs.rs/yrs/latest/yrs/#quick-start
 */

fn main() {
    //the doc is the thing in which i need to 'save' the information. idk exactly the details but any changes i make i need to apply to doc and then if im in a hypothethical scenario trying to sync from two different computers or whatever. the doc i'm storing is where i would "push" the changes to and then read from if i wanted to.
    let boss_of_yrs = BossOfYrs::new();
    let doc = &boss_of_yrs.doc;

    //creates a text block(? i think) with the "id" article if it doesn't exist.
    let text = doc.get_or_insert_text("article");

    {
        //ok from what i can tell. this is just what u need to do if u wanna begin editing the doc. like it doesn't edit it. it's just what u need to do if u wanna do it
        let mut txn = doc.transact_mut();

        //it says at index 0 put hello and at index 5 put world
        //
        // so if notion was using this library. the way they're handling it is that wherever u put ur mouse. like where u click. is where u start "putting stuff at index blabla"
        text.insert(&mut txn, 0, "hello");
        text.insert(&mut txn, 5, " world");
    }

    //text.get_string(&doc.transact())
    println!("{}", text.get_string(&doc.transact()));
}
