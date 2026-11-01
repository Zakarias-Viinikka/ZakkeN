//! Caller pov: the DOM textarea holds `old` text with the caret at
//! `caret` (a UTF-16 offset, which is what selection_start returns).
//! Something external — a reset, an undo-reset, a state sync — changed
//! the textarea's value to `new`. Return where the caret should go in
//! `new` so it stays at the same logical position.
//!
//! Internals pov: find the longest common prefix and suffix of old and
//! new in UTF-16 code units. If the caret is inside the prefix, keep it.
//! If it's inside the suffix, shift by (new_len - old_len). If it's
//! inside the changed middle, snap to the end of the prefix — that's the
//! only position that's guaranteed to still exist.
//!
//! UTF-16 code units are used rather than bytes or chars because
//! selection_start, selection_end, and set_selection_range on the DOM
//! textarea all measure in UTF-16. Getting this wrong on a block with
//! emoji or non-BMP characters puts the caret in the wrong place.

pub fn map_caret_across_edit(old: &str, new: &str, caret: u32) -> u32 {
    let old_u: Vec<u16> = old.encode_utf16().collect();
    let new_u: Vec<u16> = new.encode_utf16().collect();
    let caret = caret as usize;

    let min_len = old_u.len().min(new_u.len());
    let mut prefix = 0usize;
    while prefix < min_len && old_u[prefix] == new_u[prefix] {
        prefix += 1;
    }

    let mut suffix = 0usize;
    while suffix < (old_u.len() - prefix)
        && suffix < (new_u.len() - prefix)
        && old_u[old_u.len() - 1 - suffix] == new_u[new_u.len() - 1 - suffix]
    {
        suffix += 1;
    }

    // Start of old always maps to start of new. End of old always maps
    // to end of new. The prefix / suffix branches below would get these
    // wrong on the boundary (prefix == old_len for the end case, so
    // `caret <= prefix` would return the old length instead of the new
    // length).
    if caret == 0 {
        return 0;
    }
    if caret == old_u.len() {
        return new_u.len() as u32;
    }
    if caret <= prefix {
        return caret as u32;
    }
    if caret >= old_u.len() - suffix {
        let delta = new_u.len() as isize - old_u.len() as isize;
        let shifted = (caret as isize + delta).max(prefix as isize);
        return shifted as u32;
    }
    prefix as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_strings_keep_the_caret() {
        assert_eq!(map_caret_across_edit("hello", "hello", 3), 3);
        assert_eq!(map_caret_across_edit("hello", "hello", 0), 0);
        assert_eq!(map_caret_across_edit("hello", "hello", 5), 5);
    }

    #[test]
    fn caret_before_insertion_is_unchanged() {
        // "hello" -> "heyhello", caret at 2. Insertion is at 0.
        // Prefix = 0 ("h" vs "h"? wait, old[0]='h', new[0]='h' — prefix grows).
        // Actually "hello" vs "heyhello": prefix = "h", then "e", then old[2]='l' vs new[2]='y' differ. prefix = 2.
        // Suffix: from end, "hello" vs "hello" (last 5 of new). suffix = 5.
        // Caret = 2 <= prefix = 2 → return 2.
        assert_eq!(map_caret_across_edit("hello", "heyhello", 2), 2);
    }

    #[test]
    fn caret_after_insertion_is_shifted() {
        // "hello" -> "heyhello", caret at 4. Insertion of "hey" at position 1.
        // Wait, let me recompute. "hello" -> "heyhello":
        // h-e-l-l-o vs h-e-y-h-e-l-l-o.
        // Prefix: h-e, then l vs y differ. prefix = 2.
        // Suffix: h-e-l-l-o (5 chars), then new[2]='y' vs old[?] no, we compare old[-suffix-1] and new[-suffix-1].
        // old_u has 5, new_u has 8. suffix=5: old_u[5-1-5+1]... let me just say suffix = 5.
        // Caret = 4 >= old_u.len() - suffix = 5 - 5 = 0 → yes. Delta = 8-5 = 3. Shifted = 4+3 = 7.
        assert_eq!(map_caret_across_edit("hello", "heyhello", 4), 7);
    }

    #[test]
    fn caret_inside_the_change_snaps_to_prefix() {
        // "abcdef" -> "abXYZf", caret at 3. Changed middle.
        // Prefix: "ab" (2). Suffix: "f" (1). Caret 3 is between prefix and suffix start (5).
        // → returns 2 (prefix end).
        assert_eq!(map_caret_across_edit("abcdef", "abXYZf", 3), 2);
    }

    #[test]
    fn caret_at_end_of_old_goes_to_end_of_new() {
        // "hello" -> "hello world", caret at 5 (end of old).
        // Prefix = 5, suffix = 0. Caret 5 >= old.len() - suffix = 5. Delta = 6. Shifted = 11.
        assert_eq!(map_caret_across_edit("hello", "hello world", 5), 11);
    }

    #[test]
    fn delete_from_beginning_shifts_caret_left() {
        // "prefixhello" -> "hello", caret at 8. Deletion of "prefix".
        // Prefix = 0, suffix = 5. Caret 8 >= 11 - 5 = 6. Delta = 5 - 11 = -6. Shifted = 8 - 6 = 2.
        assert_eq!(map_caret_across_edit("prefixhello", "hello", 8), 2);
    }

    #[test]
    fn empty_old_new_nonempty_gives_zero() {
        assert_eq!(map_caret_across_edit("", "hello", 0), 0);
    }

    #[test]
    fn nonempty_old_empty_new_gives_zero() {
        assert_eq!(map_caret_across_edit("hello", "", 3), 0);
        assert_eq!(map_caret_across_edit("hello", "", 5), 0);
    }

    #[test]
    fn unicode_uses_utf16_offsets() {
        // "å" is one UTF-16 code unit but two UTF-8 bytes. Caret positions
        // at u32 come in as UTF-16 offsets, and the function works in UTF-16.
        // "åb" -> "åc", caret at 2 (after 'b').
        assert_eq!(map_caret_across_edit("åb", "åc", 2), 2);
        // "åb" -> "åabc", caret at 2 (after 'b' which is at utf16 index 1).
        // Wait, "åb": utf16 = [0xE5, 'b']. Caret at 2 = after 'b' = end.
        // "åabc": utf16 = [0xE5, 'a', 'b', 'c']. End is 4. Delta = 2. Shifted = 4.
        assert_eq!(map_caret_across_edit("åb", "åabc", 2), 4);
    }

    #[test]
    fn emoji_uses_two_utf16_units() {
        // "😀" is 1 codepoint but 2 UTF-16 code units. Caret positions in
        // the DOM are UTF-16. "a😀b" has 4 UTF-16 units, caret after 'b' is 4.
        let old = "a😀b";
        let new = "a😀bc";
        assert_eq!(map_caret_across_edit(old, new, 4), 5);
    }
}
