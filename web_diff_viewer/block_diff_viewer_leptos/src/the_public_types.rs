//! Caller pov: the types the reconcile flow hands into `DiffViewer` and
//! gets back through its `output` signal. `Block` and `LoveLetterSketch`
//! come from `generate_love_letters`. `LoveLetter` comes from the
//! letter crate.

pub use generate_love_letters::{Block, LoveLetterSketch};
pub use love_letter::LoveLetter;

/// One page's blocks plus its raw years snapshot. The blob is used when
/// the caller wants the viewer to build love letters.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PageBlocksForTheViewer {
    pub page_id: String,
    pub is_main_menu_page: bool,
    pub blocks: Vec<Block>,
    pub blob: Vec<u8>,
}

/// What `DiffViewer` writes to its `output` signal when the caller asks
/// it to finish.
///
/// - `kept_blocks`: the editable right-side blocks in their current
///   visual order.
/// - `love_letters`: the love letters that turn the server's page into
///   `kept_blocks`. For a client-only page (server = None), this is a
///   single `CreateNewPage` letter. Empty if the caller did not supply a
///   `server_page_boss` (diff-only mode).
#[derive(Clone, Default)]
pub struct ViewerOutput {
    pub kept_blocks: Vec<Block>,
    pub love_letters: Vec<LoveLetter>,
}

/// Which side wins the reconcile for one page. `Mine` puts server on
/// the left and client on the right (client editable); `Server` puts
/// client on the left and server on the right.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PickedSide {
    Mine,
    Server,
}