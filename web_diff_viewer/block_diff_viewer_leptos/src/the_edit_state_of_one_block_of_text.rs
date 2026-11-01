//! Caller pov: one block's editable text on the right side of the diff.
//! Holds the original text (what the right page said at mount), the
//! current text (what the user has typed), and the current inline
//! highlight ranges (which spans of the current text are "different
//! from the left page"). Highlight ranges are adjusted on every
//! keystroke: only the parts the user actually touches lose their
//! highlight. Typing outside a highlight does not create a new one,
//! ever.
//!
//! Invariants this struct maintains:
//! - `stashed_edit.is_some()` implies `current == original` and
//!   `current_highlights == initial_highlights`. A stash only exists
//!   right after a reset, and any typing clears it.
//! - `reset()` when `current == original` is a no-op.
//! - `undo_reset()` when `stashed_edit.is_none()` is a no-op.
//! - `user_typed()` always clears the stash, even if the typed text
//!   happens to equal the original.
//! - Highlights never grow. If a keystroke did not overlap a highlight
//!   range, that range is unchanged (possibly shifted).

use std::ops::Range;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockEditState {
    original: String,
    initial_highlights: Vec<Range<usize>>,
    current: String,
    current_highlights: Vec<Range<usize>>,
    stashed_edit: Option<(String, Vec<Range<usize>>)>,
}

impl BlockEditState {
    pub fn new(original: impl Into<String>) -> Self {
        Self::with_highlights(original, Vec::new())
    }

    pub fn with_highlights(
        original: impl Into<String>,
        initial_highlights: Vec<Range<usize>>,
    ) -> Self {
        let original = original.into();
        let current_highlights = initial_highlights.clone();
        Self {
            current: original.clone(),
            original,
            initial_highlights,
            current_highlights,
            stashed_edit: None,
        }
    }

    pub fn original(&self) -> &str {
        &self.original
    }

    pub fn current(&self) -> &str {
        &self.current
    }

    pub fn current_highlights(&self) -> &[Range<usize>] {
        &self.current_highlights
    }

    /// Called on every textarea input event. Adjusts highlights against
    /// the change (touch-inside shrinks, touch-outside is a no-op), then
    /// sets current. Clears any stash.
    pub fn user_typed(&mut self, new_text: impl Into<String>) {
        let new_text = new_text.into();
        if new_text != self.current && !self.current_highlights.is_empty() {
            self.current_highlights = adjust_highlights_for_edit(
                &self.current_highlights,
                &self.current,
                &new_text,
            );
        }
        self.current = new_text;
        self.stashed_edit = None;
    }

    pub fn reset(&mut self) {
        if self.current == self.original {
            return;
        }
        let stashed = (
            std::mem::replace(&mut self.current, self.original.clone()),
            std::mem::replace(
                &mut self.current_highlights,
                self.initial_highlights.clone(),
            ),
        );
        self.stashed_edit = Some(stashed);
    }

    pub fn undo_reset(&mut self) {
        if let Some((text, highlights)) = self.stashed_edit.take() {
            self.current = text;
            self.current_highlights = highlights;
        }
    }

    pub fn is_edited(&self) -> bool {
        self.current != self.original
    }

    pub fn can_reset(&self) -> bool {
        self.is_edited()
    }

    pub fn can_undo_reset(&self) -> bool {
        self.stashed_edit.is_some()
    }
}

/// Given a set of byte-range highlights into `old_text`, an edit that
/// turns `old_text` into `new_text`, return the new highlights over
/// `new_text`. Ranges fully before the edit are unchanged. Ranges fully
/// after are shifted by the delta. Ranges that overlap the edit are
/// trimmed from the touched side. Ranges fully covered by the edit are
/// dropped. Ranges are never extended to cover new text the user typed
/// outside them, and never extended into text that was already outside.
fn adjust_highlights_for_edit(
    highlights: &[Range<usize>],
    old_text: &str,
    new_text: &str,
) -> Vec<Range<usize>> {
    if old_text == new_text || highlights.is_empty() {
        return highlights.to_vec();
    }
    let (prefix, old_mid_end, new_mid_end) = compute_diff_region(old_text, new_text);
    let delta = new_mid_end as isize - old_mid_end as isize;

    let mut out: Vec<Range<usize>> = Vec::with_capacity(highlights.len());
    for r in highlights {
        if r.end <= prefix {
            out.push(r.clone());
        } else if r.start >= old_mid_end {
            let s = (r.start as isize + delta) as usize;
            let e = (r.end as isize + delta) as usize;
            if s < e {
                out.push(s..e);
            }
        } else {
            if r.start < prefix {
                out.push(r.start..prefix);
            }
            if r.end > old_mid_end {
                let s = new_mid_end;
                let e = (r.end as isize + delta) as usize;
                if s < e {
                    out.push(s..e);
                }
            }
        }
    }
    merge_adjacent(out)
}

/// Merge touching or overlapping ranges. When a highlight is split by an
/// edit that removes some bytes inside it, the two halves end up adjacent
/// in the new text — `[6..8]` and `[8..10]` describe the same stretch as
/// `[6..10]`. Merging keeps the highlight visually as one piece and
/// matches what a user would say ("the highlight is still there, one
/// character shorter").
fn merge_adjacent(mut ranges: Vec<Range<usize>>) -> Vec<Range<usize>> {
    ranges.sort_by_key(|r| r.start);
    let mut out: Vec<Range<usize>> = Vec::with_capacity(ranges.len());
    for r in ranges {
        match out.last_mut() {
            Some(last) if r.start <= last.end => {
                if r.end > last.end {
                    last.end = r.end;
                }
            }
            _ => out.push(r),
        }
    }
    out
}

/// The three byte offsets that describe the change: the length of the
/// common prefix, the byte just past the last old byte the edit
/// touched, and the byte just past the last new byte the edit added.
/// All three land on char boundaries because a prefix/suffix that ends
/// mid-char can only happen when the shared bytes belong to different
/// codepoints (which they don't, since UTF-8 shares its leading byte
/// length between a codepoint and any codepoint whose encoding starts
/// the same).
fn compute_diff_region(old: &str, new: &str) -> (usize, usize, usize) {
    let mut prefix = old
        .bytes()
        .zip(new.bytes())
        .take_while(|(a, b)| a == b)
        .count();
    while prefix > 0 && (!old.is_char_boundary(prefix) || !new.is_char_boundary(prefix)) {
        prefix -= 1;
    }
    let old_tail = &old[prefix..];
    let new_tail = &new[prefix..];
    let max_suffix = old_tail.len().min(new_tail.len());
    let mut suffix = 0;
    while suffix < max_suffix
        && old_tail.as_bytes()[old_tail.len() - 1 - suffix]
            == new_tail.as_bytes()[new_tail.len() - 1 - suffix]
    {
        suffix += 1;
    }
    while suffix > 0
        && (!old.is_char_boundary(old.len() - suffix)
            || !new.is_char_boundary(new.len() - suffix))
    {
        suffix -= 1;
    }
    (prefix, old.len() - suffix, new.len() - suffix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_block_is_not_edited() {
        let s = BlockEditState::new("hello");
        assert_eq!(s.original(), "hello");
        assert_eq!(s.current(), "hello");
        assert!(!s.is_edited());
        assert!(!s.can_reset());
        assert!(!s.can_undo_reset());
        assert!(s.current_highlights().is_empty());
    }

    #[test]
    fn typing_sets_current_and_marks_edited() {
        let mut s = BlockEditState::new("hello");
        s.user_typed("hello world");
        assert_eq!(s.current(), "hello world");
        assert!(s.is_edited());
        assert!(s.can_reset());
        assert!(!s.can_undo_reset());
    }

    #[test]
    fn typing_the_original_back_clears_the_edited_flag() {
        let mut s = BlockEditState::new("hello");
        s.user_typed("hello world");
        s.user_typed("hello");
        assert!(!s.is_edited());
        assert!(!s.can_reset());
        assert!(!s.can_undo_reset());
    }

    #[test]
    fn reset_stashes_and_undo_reset_restores() {
        let mut s = BlockEditState::new("hello");
        s.user_typed("goodbye");
        s.reset();
        assert_eq!(s.current(), "hello");
        assert!(!s.is_edited());
        assert!(!s.can_reset());
        assert!(s.can_undo_reset());

        s.undo_reset();
        assert_eq!(s.current(), "goodbye");
        assert!(s.is_edited());
        assert!(s.can_reset());
        assert!(!s.can_undo_reset());
    }

    #[test]
    fn the_humans_exact_sequence_reset_then_undo_then_reset() {
        let mut s = BlockEditState::new("hello");
        s.user_typed("goodbye");
        s.reset();
        assert!(!s.is_edited());
        assert!(s.can_undo_reset());

        s.undo_reset();
        assert!(s.is_edited());
        assert!(s.can_reset());

        s.reset();
        assert!(!s.is_edited());
        assert!(s.can_undo_reset());
        assert_eq!(s.current(), "hello");
    }

    #[test]
    fn reset_when_already_original_is_a_noop() {
        let mut s = BlockEditState::new("hello");
        s.reset();
        assert!(!s.is_edited());
        assert!(!s.can_undo_reset());
    }

    #[test]
    fn undo_reset_without_a_stash_is_a_noop() {
        let mut s = BlockEditState::new("hello");
        s.undo_reset();
        assert_eq!(s.current(), "hello");
    }

    #[test]
    fn typing_after_reset_clears_the_stash() {
        let mut s = BlockEditState::new("hello");
        s.user_typed("goodbye");
        s.reset();
        assert!(s.can_undo_reset());

        s.user_typed("totally different");
        assert!(!s.can_undo_reset(), "typing should have cleared the stash");
        assert_eq!(s.current(), "totally different");
    }

    #[test]
    fn multiple_resets_keep_the_most_recent_stash() {
        let mut s = BlockEditState::new("hello");
        s.user_typed("first");
        s.reset();
        s.undo_reset();
        s.user_typed("second");
        s.reset();
        s.undo_reset();
        assert_eq!(s.current(), "second");
    }

    #[test]
    fn unicode_round_trip() {
        let mut s = BlockEditState::new("åäö");
        s.user_typed("åäö först");
        s.reset();
        s.undo_reset();
        assert_eq!(s.current(), "åäö först");
        assert!(s.is_edited());
    }

    // ---- highlight tracking ----

    #[test]
    fn typing_outside_a_highlight_does_not_touch_it() {
        // "Hello world" with "world" highlighted at [6, 11]. Type at
        // the end. The highlight stays where it was.
        let mut s = BlockEditState::with_highlights("Hello world", vec![6..11]);
        s.user_typed("Hello world!");
        assert_eq!(s.current_highlights(), &[6..11]);
    }

    #[test]
    fn typing_before_a_highlight_shifts_it() {
        // "Hello world" with "world" at [6, 11]. Type "Hey " at the
        // start. The highlight should be at [10, 15] now.
        let mut s = BlockEditState::with_highlights("Hello world", vec![6..11]);
        s.user_typed("Hey Hello world");
        assert_eq!(s.current_highlights(), &[10..15]);
    }

    #[test]
    fn deleting_inside_a_highlight_shrinks_it() {
        // "Hello world" with "world" at [6, 11]. Delete the 'r' at
        // index 8. Highlight becomes [6, 10].
        let mut s = BlockEditState::with_highlights("Hello world", vec![6..11]);
        s.user_typed("Hello wold");
        assert_eq!(s.current_highlights(), &[6..10]);
    }

    #[test]
    fn deleting_the_whole_highlight_removes_it() {
        let mut s = BlockEditState::with_highlights("Hello world", vec![6..11]);
        s.user_typed("Hello ");
        assert!(s.current_highlights().is_empty());
    }

    #[test]
    fn typing_does_not_create_new_highlights() {
        // No initial highlights — typing anywhere must not add any.
        let mut s = BlockEditState::with_highlights("Hello world", vec![]);
        s.user_typed("Some entirely new text here");
        assert!(s.current_highlights().is_empty());
    }

    #[test]
    fn typing_inside_a_highlight_splits_it() {
        // "0123456789" with [2, 8] highlighted. Insert "X" at index 5.
        // Expected: [2, 5] and [6, 9] (X is not highlighted, since the
        // user typed X and the original wasn't there).
        let mut s = BlockEditState::with_highlights("0123456789", vec![2..8]);
        s.user_typed("01234X56789");
        assert_eq!(s.current_highlights(), &[2..5, 6..9]);
    }

    #[test]
    fn reset_restores_initial_highlights() {
        let mut s = BlockEditState::with_highlights("Hello world", vec![6..11]);
        s.user_typed("Hello world changed a lot");
        s.reset();
        assert_eq!(s.current_highlights(), &[6..11]);
    }

    #[test]
    fn undo_reset_restores_pre_reset_highlights() {
        let mut s = BlockEditState::with_highlights("Hello world", vec![6..11]);
        s.user_typed("Hello wold");
        assert_eq!(s.current_highlights(), &[6..10]);
        s.reset();
        assert_eq!(s.current_highlights(), &[6..11]);
        s.undo_reset();
        assert_eq!(s.current_highlights(), &[6..10]);
    }

    #[test]
    fn unicode_prefix_edit_keeps_highlights_on_char_boundaries() {
        // "åäö" with the whole thing highlighted. Replace 'ä' with
        // 'ö' — the byte prefix would be 0xC3 for both, but the round
        // down keeps it at a char boundary.
        let mut s = BlockEditState::with_highlights("åäö", vec![0..6]);
        s.user_typed("åöö");
        // Check that no range lands mid-char.
        for r in s.current_highlights() {
            assert!(s.current().is_char_boundary(r.start));
            assert!(s.current().is_char_boundary(r.end));
        }
    }
}