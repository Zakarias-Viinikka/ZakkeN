//! Caller pov: `block_texts_from_snapshot(bytes)` gives the ordered
//! block texts of the page encoded in `bytes`.
//!
//! Both sides call this. The server uses the output for fingerprinting.
//! The client uses the same output as the diff viewer's left and right
//! page inputs, and for its own fingerprinting. Because it's the same
//! function, what the server hashes equals what the user sees.

use std::sync::Arc;

use my_yrs_lib::{doc_from_snapshot, BossOfYrs, YrsError};

/// Ordered block texts of the page encoded in `snapshot`.
///
/// Order comes from the doc's existing (frac_position, block_id) sort,
/// the same one `get_entire_page` uses, so client and server agree.
pub fn block_texts_from_snapshot(snapshot: Vec<u8>) -> Result<Vec<String>, YrsError> {
    let boss: Arc<BossOfYrs> = Arc::new(doc_from_snapshot(
        snapshot,
        String::new(),
        String::new(),
        String::new(),
    )?);
    let blocks = boss.get_entire_page()?;
    Ok(blocks.into_iter().map(|b| b.text).collect())
}
