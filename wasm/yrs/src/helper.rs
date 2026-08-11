use crate::yrs_wrapper::{self, BossOfYrs, TextEdit};
use yrs::block::Item;
use yrs::types::ToJson;
use yrs::types::TypeRef::XmlText;
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use yrs::{
    Any, Array, ArrayRef, Doc, GetString, Map, MapPrelim, ReadTxn, Text, TextPrelim, Transact,
    Update, XmlElementPrelim, XmlElementRef, XmlTextPrelim,
};

pub fn do_insert(
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

pub fn do_delete(
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

pub fn do_replace(
    text_to_insert: &str,
    position_to_delete_at: usize,
    length: usize,
    boss_of_yrs: &mut BossOfYrs,
    page_id: usize,
    text_block_id: &str,
) -> Result<(), String> {
    do_delete(
        position_to_delete_at,
        length,
        boss_of_yrs,
        page_id,
        text_block_id,
    )?;
    do_insert(
        text_to_insert,
        position_to_delete_at,
        boss_of_yrs,
        page_id,
        text_block_id,
    )?;
    Ok(())
}

/*
*
* pub fn insert_new_block(
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
*/
