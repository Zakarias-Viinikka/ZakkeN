//! Key decisions. Pure Rust, no JS, no DOM. All offsets are UTF-16 code units
//! (what CodeMirror reports).
//!
//! JS reports a key press; `decide` says what it means and whether to consume it.
//! `None` means: not ours, let CodeMirror handle the key normally.
//!
//! Parity with the old textarea code: Enter never splits a title, and never splits an empty or
//! whitespace-only block. In those cases it is NOT consumed (the editor inserts the newline) and
//! the caller marks `was_enter` for the next input event.

#[derive(Clone, Debug, PartialEq)]
pub enum Intent {
    NavUp,
    NavDown,
    Split { before: String, after: String },
    /// Backspace at position 0, or Delete in an empty block.
    MergeUp,
    /// Enter on a title, or in an empty / whitespace-only block: not consumed, CM inserts the
    /// newline, and the next input event carries `was_enter`.
    EnterWithoutSplit,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Decision {
    pub intent: Intent,
    /// true => Rust handled it, CM must not.
    pub consume: bool,
}

fn consume(intent: Intent) -> Option<Decision> {
    Some(Decision { intent, consume: true })
}

fn enter_without_split() -> Option<Decision> {
    Some(Decision {
        intent: Intent::EnterWithoutSplit,
        consume: false,
    })
}

/// Split `text` around the UTF-16 range `from..to` (the selection is dropped).
pub fn split_utf16(text: &str, from: usize, to: usize) -> (String, String) {
    let units: Vec<u16> = text.encode_utf16().collect();
    let from = from.min(units.len());
    let to = to.clamp(from, units.len());
    (
        String::from_utf16_lossy(&units[..from]),
        String::from_utf16_lossy(&units[to..]),
    )
}

/// `text` is only called when the decision needs the document (Enter on a non-title).
pub fn decide(
    key: &str,
    anchor: usize,
    head: usize,
    doc_len: usize,
    ranges: usize,
    is_title: bool,
    text: impl FnOnce() -> String,
) -> Option<Decision> {
    if ranges != 1 {
        return None; // multi-cursor: leave it to CM
    }
    let empty = anchor == head;
    match key {
        "ArrowUp" if empty && head == 0 => consume(Intent::NavUp),
        "ArrowDown" if empty && head == doc_len => consume(Intent::NavDown),
        "Enter" if is_title || doc_len == 0 => enter_without_split(),
        "Enter" => {
            let full = text();
            if full.trim().is_empty() {
                return enter_without_split();
            }
            let (before, after) = split_utf16(&full, anchor.min(head), anchor.max(head));
            consume(Intent::Split { before, after })
        }
        "Backspace" if empty && head == 0 => consume(Intent::MergeUp),
        "Delete" if doc_len == 0 => consume(Intent::MergeUp),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(key: &str, a: usize, h: usize, len: usize, ranges: usize, t: &str) -> Option<Decision> {
        decide(key, a, h, len, ranges, false, || t.to_string())
    }

    fn dt(key: &str, a: usize, h: usize, len: usize, t: &str) -> Option<Decision> {
        decide(key, a, h, len, 1, true, || t.to_string())
    }

    #[test]
    fn arrows_only_at_edges() {
        assert_eq!(d("ArrowUp", 0, 0, 5, 1, "hello").unwrap().intent, Intent::NavUp);
        assert!(d("ArrowUp", 2, 2, 5, 1, "hello").is_none());
        assert_eq!(d("ArrowDown", 5, 5, 5, 1, "hello").unwrap().intent, Intent::NavDown);
        assert!(d("ArrowDown", 0, 5, 5, 1, "hello").is_none()); // selection
    }

    #[test]
    fn enter_splits_with_emoji() {
        // "a😀b": 😀 is 2 UTF-16 units, caret after it = 3
        let dec = d("Enter", 3, 3, 4, 1, "a😀b").unwrap();
        assert!(dec.consume);
        assert_eq!(
            dec.intent,
            Intent::Split { before: "a😀".into(), after: "b".into() }
        );
    }

    #[test]
    fn enter_on_empty_not_consumed() {
        let dec = d("Enter", 0, 0, 0, 1, "").unwrap();
        assert_eq!(dec.intent, Intent::EnterWithoutSplit);
        assert!(!dec.consume);
    }

    #[test]
    fn enter_on_whitespace_only_not_consumed() {
        let dec = d("Enter", 1, 1, 2, 1, "  ").unwrap();
        assert_eq!(dec.intent, Intent::EnterWithoutSplit);
        assert!(!dec.consume);
    }

    #[test]
    fn enter_on_title_never_splits() {
        let dec = dt("Enter", 2, 2, 5, "hello").unwrap();
        assert_eq!(dec.intent, Intent::EnterWithoutSplit);
        assert!(!dec.consume);
    }

    #[test]
    fn enter_drops_selection() {
        let dec = d("Enter", 1, 4, 6, 1, "abcdef").unwrap();
        assert_eq!(
            dec.intent,
            Intent::Split { before: "a".into(), after: "ef".into() }
        );
    }

    #[test]
    fn backspace_and_delete() {
        assert_eq!(d("Backspace", 0, 0, 3, 1, "abc").unwrap().intent, Intent::MergeUp);
        assert!(d("Backspace", 1, 1, 3, 1, "abc").is_none());
        assert_eq!(d("Delete", 0, 0, 0, 1, "").unwrap().intent, Intent::MergeUp);
        assert!(d("Delete", 0, 0, 3, 1, "abc").is_none());
    }

    #[test]
    fn multi_cursor_ignored() {
        assert!(d("Backspace", 0, 0, 3, 2, "abc").is_none());
    }
}