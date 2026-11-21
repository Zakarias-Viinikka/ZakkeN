//! Markdown editor, blackboxed.
//!
//! Depends on `cm_bridge.js` (same folder). It is copied to the dist root and
//! loaded as a classic script from `index.html`, so the global `cmBridge`
//! exists before the wasm starts.
mod markdown_conversion;

pub use markdown_conversion::{MarkdownConversion, MarkdownInputEvent};
