//! Pure edit logic. No JS, no DOM. Offsets are UTF-16 code units in the OLD document
//! (what CodeMirror reports).
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Change {
    pub from: usize,
    pub to: usize,
    pub insert: String,
}

/// Apply CodeMirror's change list (all positions in OLD-document coordinates) to `old`.
pub fn apply_changes(old: &str, changes: &[Change]) -> String {
    let units: Vec<u16> = old.encode_utf16().collect();
    let mut sorted: Vec<&Change> = changes.iter().collect();
    sorted.sort_by_key(|c| c.from);

    let mut out: Vec<u16> = Vec::with_capacity(units.len());
    let mut pos = 0usize;
    for c in sorted {
        let from = c.from.clamp(pos, units.len());
        let to = c.to.clamp(from, units.len());
        out.extend_from_slice(&units[pos..from]);
        out.extend(c.insert.encode_utf16());
        pos = to;
    }
    out.extend_from_slice(&units[pos..]);
    String::from_utf16_lossy(&out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ch(from: usize, to: usize, insert: &str) -> Change {
        Change { from, to, insert: insert.to_string() }
    }

    #[test]
    fn insert_delete_replace() {
        assert_eq!(apply_changes("hello", &[ch(5, 5, "!")]), "hello!");
        assert_eq!(apply_changes("hello", &[ch(1, 3, "")]), "hlo");
        assert_eq!(apply_changes("hello", &[ch(0, 5, "bye")]), "bye");
    }

    #[test]
    fn multi_change_old_coordinates() {
        // both positions refer to "abcdef"
        assert_eq!(apply_changes("abcdef", &[ch(0, 1, "X"), ch(4, 6, "YZW")]), "XbcdYZW");
    }

    #[test]
    fn emoji_utf16() {
        // "a😀b": 😀 occupies UTF-16 units 1..3
        assert_eq!(apply_changes("a😀b", &[ch(1, 3, "")]), "ab");
        assert_eq!(apply_changes("a😀b", &[ch(3, 3, "😁")]), "a😀😁b");
    }
}
