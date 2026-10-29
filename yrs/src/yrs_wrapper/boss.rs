use std::sync::{Arc, RwLock};

use yrs::{
    Array, Doc, GetString, In, Map, MapPrelim, ReadTxn, StateVector, Transact, XmlTextPrelim,
    XmlTextRef,
};

use super::apply_update::apply_update_bytes;
use super::block_lookup::{find_block, frac_position_by_id, read_block_at, string_field};
use super::frac_position::{compute_new_frac_position_in, get_new_frac_position};
use super::keys::{
    BLOCKS_KEY, CONTENT_KEY, FRAC_POSITION_KEY, ID_KEY, META_KEY, generate_unique_key,
};
use super::text_edit::apply_edit;
use super::types::{Block, EditTarget, PositionToInsert, TextEdit};
use crate::yrs_error::YrsError;

#[derive(uniffi::Object)]
pub struct BossOfYrs {
    pub doc: RwLock<Doc>,
    pub page_id: String,
    pub user_id: String,
    pub time: String,
}

fn insert_block_into_doc(
    doc: &Doc,
    block_id: String,
    block_content: String,
    block_meta_data: String,
    position_to_insert: PositionToInsert,
) -> Result<(), YrsError> {
    let array = doc.get_or_insert_array(BLOCKS_KEY.to_string());
    let mut txn = doc.transact_mut();
    let len = array.len(&txn) as u32;
    let frac_position = match &position_to_insert {
        PositionToInsert::AtEnd => get_new_frac_position(&array, &txn, len, len)?,
        PositionToInsert::Between {
            block_id_of_above,
            block_id_of_below,
        } => compute_new_frac_position_in(
            &array,
            &txn,
            block_id_of_above.as_deref(),
            block_id_of_below.as_deref(),
        )?,
    };
    let text_as_xml_text_ref = XmlTextPrelim::new(block_content);
    array.insert(
        &mut txn,
        len,
        MapPrelim::from([
            (ID_KEY.to_string(), In::from(block_id)),
            (CONTENT_KEY.to_string(), In::from(text_as_xml_text_ref)),
            (META_KEY.to_string(), In::from(block_meta_data)),
            (FRAC_POSITION_KEY.to_string(), In::from(frac_position)),
        ]),
    );
    Ok(())
}

#[uniffi::export]
impl BossOfYrs {
    #[uniffi::constructor]
    pub fn new(user_id: String, unix_time: String) -> Self {
        let unique_id = generate_unique_key(&user_id, &unix_time);
        Self {
            doc: RwLock::new(Doc::new()),
            page_id: unique_id,
            user_id,
            time: unix_time,
        }
    }

    pub fn page_id(self: Arc<Self>) -> String {
        self.page_id.clone()
    }

    pub fn user_id(self: Arc<Self>) -> String {
        self.user_id.clone()
    }

    pub fn insert_new_block(
        self: Arc<Self>,
        block_content: String,
        block_meta_data: String,
        position: PositionToInsert,
    ) -> Result<String, YrsError> {
        crate::guarded!("insert_new_block", move || {
            let doc = crate::read_doc!(self.doc, "insert_new_block");
            let block_id = generate_unique_key(&self.user_id, &self.time);
            insert_block_into_doc(
                &doc,
                block_id.clone(),
                block_content,
                block_meta_data,
                position,
            )?;
            Ok(block_id)
        })
    }

    pub fn insert_new_block_with_preexisting_id(
        self: Arc<Self>,
        block_id: String,
        block_content: String,
        block_meta_data: String,
        position_to_insert: PositionToInsert,
    ) -> Result<(), YrsError> {
        crate::guarded!("insert_new_block_with_preexisting_id", move || {
            let doc = crate::read_doc!(self.doc, "insert_new_block_with_preexisting_id");
            {
                let array = doc.get_or_insert_array(BLOCKS_KEY.to_string());
                let txn = doc.transact();
                if find_block(&array, &txn, &block_id).is_ok() {
                    return Err(crate::generic_error!(
                        format!("block id {block_id} already exists in this page"),
                        "insert_new_block_with_preexisting_id"
                    ));
                }
            }
            insert_block_into_doc(
                &doc,
                block_id,
                block_content,
                block_meta_data,
                position_to_insert,
            )
        })
    }

    pub fn get_entire_page(self: Arc<Self>) -> Result<Vec<Block>, YrsError> {
        crate::guarded!("get_entire_page", move || {
            let doc = crate::read_doc!(self.doc, "get_entire_page");
            let array = doc.get_or_insert_array(BLOCKS_KEY.to_string());
            let txn = doc.transact();
            let len = array.len(&txn) as usize;
            let mut with_keys: Vec<(String, Block)> = Vec::with_capacity(len);
            for position in 0..len {
                with_keys.push(read_block_at(&array, &txn, position)?);
            }
            with_keys.sort_by(|a, b| {
                a.0.cmp(&b.0)
                    .then_with(|| a.1.id_in_yrs.cmp(&b.1.id_in_yrs))
            });
            Ok(with_keys.into_iter().map(|(_, b)| b).collect())
        })
    }

    pub fn show_doc_info(self: Arc<Self>) -> Result<(), YrsError> {
        use yrs::types::ToJson;
        crate::guarded!("show_doc_info", move || {
            let doc = crate::read_doc!(self.doc, "show_doc_info");
            let array = doc.get_or_insert_array(BLOCKS_KEY.to_string());
            let txn = doc.transact();
            let json_representation = array.to_json(&txn);
            println!("data blocks represented as json:");
            println!("{}", json_representation);
            Ok(())
        })
    }

    pub fn edit_text_block(
        self: Arc<Self>,
        block_id: String,
        text_edit: TextEdit,
        edit_target: EditTarget,
    ) -> Result<(), YrsError> {
        crate::guarded!("edit_text_block_insert", move || {
            let doc = crate::write_doc!(self.doc, "edit_text_block_insert");
            let block = {
                let array = doc.get_or_insert_array(BLOCKS_KEY.to_string());
                let txn = doc.transact();
                find_block(&array, &txn, &block_id)
                    .map(|(_, m)| m)
                    .map_err(|_| {
                        crate::generic_error!(
                            format!("found no block with id: {block_id}"),
                            "edit_text_block_insert"
                        )
                    })?
            };
            apply_edit(&doc, block, text_edit, edit_target)
        })
    }

    pub fn delete_block(self: Arc<Self>, block_id: String) -> Result<(), YrsError> {
        let do_delete = move || -> Result<(), YrsError> {
            let doc = crate::write_doc!(self.doc, "delete_block");
            let array = doc.get_or_insert_array(BLOCKS_KEY.to_string());
            let position = {
                let txn = doc.transact();
                find_block(&array, &txn, &block_id)
                    .map(|(i, _)| i)
                    .map_err(|_| {
                        crate::generic_error!(
                            format!("found no block with id: {block_id}"),
                            "delete_block"
                        )
                    })?
            };
            let mut txn = doc.transact_mut();
            array.remove(&mut txn, position);
            Ok(())
        };
        crate::guarded!("delete_block", do_delete)
    }

    pub fn snapshot(self: Arc<Self>) -> Result<Vec<u8>, YrsError> {
        crate::guarded!("snapshot", move || {
            let doc = crate::read_doc!(self.doc, "snapshot");
            Ok(doc.transact().encode_diff_v1(&StateVector::default()))
        })
    }

    pub fn merge_with_snapshot(self: Arc<Self>, snapshot: Vec<u8>) -> Result<(), YrsError> {
        crate::guarded!("merge_with_snapshot", move || {
            let doc = crate::write_doc!(self.doc, "merge_with_snapshot");
            apply_update_bytes(&doc, &snapshot, "merge_with_snapshot")
        })
    }

    pub fn merge_with(self: Arc<Self>, other: Arc<BossOfYrs>) -> Result<(), YrsError> {
        crate::guarded!("merge_with", move || {
            let sv = {
                let doc = crate::read_doc!(self.doc, "merge_with");
                doc.transact().state_vector()
            };
            let diff = {
                let other_doc = crate::read_doc!(other.doc, "merge_with");
                other_doc.transact().encode_diff_v1(&sv)
            };
            let doc = crate::write_doc!(self.doc, "merge_with");
            apply_update_bytes(&doc, &diff, "merge_with")
        })
    }

    pub fn move_block(
        self: Arc<Self>,
        block_id: String,
        block_id_of_above: Option<String>,
        block_id_of_below: Option<String>,
    ) -> Result<(), YrsError> {
        if block_id_of_above.as_deref() == Some(block_id.as_str()) {
            return Err(crate::generic_error!(
                "move_block: block_id_of_above equals block_id",
                "move_block"
            ));
        }
        if block_id_of_below.as_deref() == Some(block_id.as_str()) {
            return Err(crate::generic_error!(
                "move_block: block_id_of_below equals block_id",
                "move_block"
            ));
        }

        crate::guarded!("move_block", move || {
            let doc = crate::write_doc!(self.doc, "move_block");
            let array = doc.get_or_insert_array(BLOCKS_KEY.to_string());
            let mut txn = doc.transact_mut();
            let (_, moving_map) = find_block(&array, &txn, &block_id)?;
            let new_frac = compute_new_frac_position_in(
                &array,
                &txn,
                block_id_of_above.as_deref(),
                block_id_of_below.as_deref(),
            )?;
            moving_map.insert(&mut txn, FRAC_POSITION_KEY.to_string(), In::from(new_frac));
            Ok(())
        })
    }

    pub fn get_frac_position_of_block(
        self: Arc<Self>,
        block_id: String,
    ) -> Result<String, YrsError> {
        crate::guarded!("get_frac_position_of_block", move || {
            let doc = crate::read_doc!(self.doc, "get_frac_position_of_block");
            let array = doc.get_or_insert_array(BLOCKS_KEY.to_string());
            let txn = doc.transact();
            frac_position_by_id(&array, &txn, &block_id)
        })
    }

    pub fn read_block_meta(self: Arc<Self>, block_id: String) -> Result<Option<String>, YrsError> {
        crate::guarded!("read_block_meta", move || {
            let doc = crate::read_doc!(self.doc, "read_block_meta");
            let block = {
                let array = doc.get_or_insert_array(BLOCKS_KEY.to_string());
                let txn = doc.transact();
                find_block(&array, &txn, &block_id).ok().map(|(_, m)| m)
            };
            match block {
                None => Ok(None),
                Some(block_map) => {
                    let txn = doc.transact();
                    let meta = string_field(&block_map, &txn, META_KEY).ok_or_else(|| {
                        crate::generic_error!("read_block_meta: no meta field", "read_block_meta")
                    })?;
                    Ok(Some(meta))
                }
            }
        })
    }

    pub fn read_block(self: Arc<Self>, block_id: String) -> Result<Option<String>, YrsError> {
        crate::guarded!("read_block", move || {
            let doc = crate::read_doc!(self.doc, "read_block");
            let block = {
                let array = doc.get_or_insert_array(BLOCKS_KEY.to_string());
                let txn = doc.transact();
                find_block(&array, &txn, &block_id).ok().map(|(_, m)| m)
            };
            match block {
                None => Ok(None),
                Some(block_map) => {
                    let txn = doc.transact();
                    let text = block_map
                        .get(&txn, CONTENT_KEY)
                        .ok_or_else(|| {
                            crate::generic_error!("read_block: no content field", "read_block")
                        })?
                        .cast::<XmlTextRef>()
                        .map_err(|_| {
                            crate::generic_error!(
                                "read_block: failed to cast content field to XmlTextRef",
                                "read_block"
                            )
                        })?
                        .get_string(&txn);
                    Ok(Some(text))
                }
            }
        })
    }
}
