use yrs::updates::decoder::Decode;
use yrs::{
    Array, Doc, GetString, In, Map, MapPrelim, MapRef, OffsetKind, Options, ReadTxn, StateVector,
    Text, Transact, Update, XmlTextPrelim, XmlTextRef,
};

use crate::edit_translation::{TextEdit, utf16_len};

const BLOCKS_KEY: &str = "blocks";
const ID_KEY: &str = "id";
const TEXT_KEY: &str = "text";

/// Authority storage for one block, mirroring the real project's structure:
/// Doc -> Array "blocks" -> Map { "id": String, "text": XmlTextRef }.
/// Uses `OffsetKind::Utf16` so CodeMirror's UTF-16 positions and lengths pass straight through.
pub struct AuthorityBlockDoc {
    doc: Doc,
    block_id: String,
}

impl AuthorityBlockDoc {
    pub fn new(block_id: impl Into<String>, initial_text: &str) -> Self {
        let block_id = block_id.into();
        let doc = new_utf16_doc();
        {
            let array = doc.get_or_insert_array(BLOCKS_KEY);
            let mut txn = doc.transact_mut();
            let xml_text = XmlTextPrelim::new(initial_text.to_string());
            array.insert(
                &mut txn,
                0,
                MapPrelim::from([
                    (ID_KEY.to_string(), In::from(block_id.clone())),
                    (TEXT_KEY.to_string(), In::from(xml_text)),
                ]),
            );
        }
        Self { doc, block_id }
    }

    pub fn block_id(&self) -> &str {
        &self.block_id
    }

    pub fn read_text(&self) -> Result<String, String> {
        let array = self.doc.get_or_insert_array(BLOCKS_KEY);
        let txn = self.doc.transact();
        let map = find_block_map(&array, &txn, &self.block_id)?;
        let text_ref = get_text_ref(&map, &txn)?;
        Ok(text_ref.get_string(&txn))
    }

    /// Applies `TextEdit`s (expected in descending old-doc UTF-16 position order)
    /// inside a single yrs transaction. Delete/Replace lengths count UTF-16 code units
    /// to match `OffsetKind::Utf16`.
    pub fn apply_edits(&self, edits_desc: &[TextEdit]) -> Result<(), String> {
        let array = self.doc.get_or_insert_array(BLOCKS_KEY);
        let mut txn = self.doc.transact_mut();
        let map = find_block_map(&array, &txn, &self.block_id)?;
        let text_ref = get_text_ref(&map, &txn)?;

        for edit in edits_desc {
            match edit {
                TextEdit::Insert { text, position } => {
                    text_ref.insert(&mut txn, *position, text);
                }
                TextEdit::Delete { text, position } => {
                    text_ref.remove_range(&mut txn, *position, utf16_len(text));
                }
                TextEdit::Replace {
                    old_text,
                    new_text,
                    position,
                } => {
                    text_ref.remove_range(&mut txn, *position, utf16_len(old_text));
                    text_ref.insert(&mut txn, *position, new_text);
                }
            }
        }
        Ok(())
    }

    /// Simulates a remote peer: forks a second UTF-16 Doc from our snapshot,
    /// runs `peer_edit` on the peer's `XmlTextRef`, encodes the diff against our
    /// state vector, merges it into our Doc, and returns the resulting string.
    pub fn simulate_remote_edit<F>(&self, peer_edit: F) -> Result<String, String>
    where
        F: FnOnce(&mut yrs::TransactionMut<'_>, &XmlTextRef),
    {
        let (snapshot, sv) = {
            let txn = self.doc.transact();
            (
                txn.encode_diff_v1(&StateVector::default()),
                txn.state_vector(),
            )
        };

        let peer = new_utf16_doc();
        {
            let update = Update::decode_v1(&snapshot).map_err(|e| format!("decode snapshot: {e}"))?;
            let mut txn = peer.transact_mut();
            txn.apply_update(update).map_err(|e| format!("apply snapshot: {e}"))?;
        }
        {
            let array = peer.get_or_insert_array(BLOCKS_KEY);
            let mut txn = peer.transact_mut();
            let map = find_block_map(&array, &txn, &self.block_id)?;
            let text_ref = get_text_ref(&map, &txn)?;
            peer_edit(&mut txn, &text_ref);
        }

        let diff = {
            let txn = peer.transact();
            txn.encode_diff_v1(&sv)
        };

        {
            let update = Update::decode_v1(&diff).map_err(|e| format!("decode diff: {e}"))?;
            let mut txn = self.doc.transact_mut();
            txn.apply_update(update).map_err(|e| format!("apply diff: {e}"))?;
        }

        self.read_text()
    }
}

fn new_utf16_doc() -> Doc {
    let mut opts = Options::default();
    opts.offset_kind = OffsetKind::Utf16;
    Doc::with_options(opts)
}

fn find_block_map<T: ReadTxn>(
    array: &yrs::ArrayRef,
    txn: &T,
    target_id: &str,
) -> Result<MapRef, String> {
    let len = array.len(txn);
    for idx in 0..len {
        if let Some(yrs::Out::YMap(map)) = array.get(txn, idx) {
            if let Some(yrs::Out::Any(yrs::Any::String(id))) = map.get(txn, ID_KEY) {
                if id.as_ref() == target_id {
                    return Ok(map);
                }
            }
        }
    }
    Err(format!("block {target_id} not found in authority doc"))
}

fn get_text_ref<T: ReadTxn>(map: &MapRef, txn: &T) -> Result<XmlTextRef, String> {
    map.get(txn, TEXT_KEY)
        .ok_or_else(|| "missing text field in block map".to_string())?
        .cast::<XmlTextRef>()
        .map_err(|_| "text field is not an XmlTextRef".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edit_translation::{CmChange, convert_changes_to_edits_desc};

    #[test]
    fn discover_default_yrs_offset_unit_is_bytes() {
        // Verify what unit `Doc::new()` (default Options) uses for XmlTextRef::insert / remove_range.
        let doc = Doc::new();
        assert_eq!(doc.offset_kind(), OffsetKind::Bytes);

        let array = doc.get_or_insert_array(BLOCKS_KEY);
        {
            let mut txn = doc.transact_mut();
            array.insert(
                &mut txn,
                0,
                MapPrelim::from([
                    (ID_KEY.to_string(), In::from("b1")),
                    (TEXT_KEY.to_string(), In::from(XmlTextPrelim::new("😀日b"))),
                ]),
            );
        }

        // "😀" is 4 UTF-8 bytes, "日" is 3 UTF-8 bytes, "b" is 1 byte.
        // Removing "日" in default Doc::new() requires byte offset 4, byte length 3.
        let mut txn = doc.transact_mut();
        let map = find_block_map(&array, &txn, "b1").unwrap();
        let text_ref = get_text_ref(&map, &txn).unwrap();
        text_ref.remove_range(&mut txn, 4, 3);
        assert_eq!(text_ref.get_string(&txn), "😀b");

        // Inserting at byte offset 4 puts text right after "😀".
        text_ref.insert(&mut txn, 4, "X");
        assert_eq!(text_ref.get_string(&txn), "😀Xb");
    }

    #[test]
    fn utf16_doc_accepts_cm_offsets_for_emoji_zwj_and_cjk() {
        let initial = "a😀👨‍👩‍👧‍👦日éb";
        let auth = AuthorityBlockDoc::new("b1", initial);

        // UTF-16 layout:
        // a: 0..1 (1)
        // 😀: 1..3 (2)
        // 👨‍👩‍👧‍👦: 3..14 (👨=2 + ZWJ=1 + 👩=2 + ZWJ=1 + 👧=2 + ZWJ=1 + 👦=2 => 11 UTF-16 units)
        // 日: 14..15 (1)
        // é: 15..16 (1)
        // b: 16..17 (1)
        let changes = vec![
            CmChange {
                from: 1,
                to: 3,
                insert: "🐱".to_string(),
            },
            CmChange {
                from: 14,
                to: 15,
                insert: "".to_string(),
            },
            CmChange {
                from: 17,
                to: 17,
                insert: "!".to_string(),
            },
        ];

        let edits = convert_changes_to_edits_desc(initial, &changes).unwrap();
        auth.apply_edits(&edits).unwrap();
        assert_eq!(auth.read_text().unwrap(), "a🐱👨‍👩‍👧‍👦éb!");
    }

    #[test]
    fn remote_simulation_syncs_via_state_vector_diff() {
        let auth = AuthorityBlockDoc::new("b1", "hello world");
        let after = auth
            .simulate_remote_edit(|txn, text_ref| {
                text_ref.insert(txn, 0, "[remote] ");
            })
            .unwrap();
        assert_eq!(after, "[remote] hello world");
        assert_eq!(auth.read_text().unwrap(), "[remote] hello world");
    }
}
