//! Caller pov: the types the reconcile flow hands into `DiffViewer` and
//! gets back through its `output` signal. `Block` and
//! `LoveLetterSketch` come from `generate_love_letters`.

pub use generate_love_letters::{Block, LoveLetterSketch};

/// One page's blocks, with everything the viewer needs to render a side,
/// generate sketches on finish, and (for a client-only page) emit a
/// `CreateNewPage` sketch.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PageBlocksForTheViewer {
    pub page_id: String,
    pub is_main_menu_page: bool,
    pub blocks: Vec<Block>,
}

/// What `DiffViewer` writes to its `output` signal when the caller asks
/// it to finish. `kept_blocks` is the editable right-side blocks in
/// their current visual order; `sketches` are the love-letter sketches
/// that would turn the server's page into that list.
#[derive(Clone, Default)]
pub struct ViewerOutput {
    pub kept_blocks: Vec<Block>,
    pub sketches: Vec<LoveLetterSketch>,
}

/// Which side wins the reconcile for one page. `Mine` = the client's
/// local version is kept (server goes on the visual left, client on the
/// right). `Server` = the server's version is kept (client on the left,
/// server on the right). Either way, the right side is editable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PickedSide {
    Mine,
    Server,
}
