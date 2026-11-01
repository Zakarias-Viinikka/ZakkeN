//! Caller pov: pass a line's length (in bytes) and the byte ranges that
//! should be highlighted, and get back a list of segments that cover
//! the line with no gaps. Each segment is (start, end, is_highlighted).
//! Every segment boundary is either the line's start, the line's end,
//! or an edge of a highlight range, so no segment straddles a highlight
//! boundary.
//!
//! Internals pov: cut every highlight edge into the cuts list, plus 0
//! and len. Sort and dedup. Walk consecutive pairs. A segment is
//! highlighted iff it lies entirely inside some highlight range (which
//! is guaranteed because every highlight edge is a cut).
//!
//! The byte boundaries are taken as given. `inline_spans` from
//! block_text_diff always produces char boundaries; if a caller hand-
//! rolls ranges and gets a mid-codepoint boundary, `text.get(range)`
//! will fail and the caller should draw plain. `render_segments_for_text`
//! in this crate does that check.

use std::ops::Range;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Segment {
    pub start: usize,
    pub end: usize,
    pub is_highlighted: bool,
}

pub fn segs_for_a_line(len: usize, highlights: &[Range<usize>]) -> Vec<Segment> {
    if len == 0 {
        return Vec::new();
    }
    // Merge overlapping ranges first so overlapping input does not
    // produce extra cuts. Otherwise [2..6, 4..8] yields five segments
    // (0..2, 2..4, 4..6, 6..8, 8..10) instead of the expected three
    // (0..2, 2..8, 8..10) — the merge turns "overlapping" into "one
    // stretch" as the docs above say.
    let mut merged: Vec<Range<usize>> = Vec::with_capacity(highlights.len());
    let mut sorted: Vec<Range<usize>> = highlights
        .iter()
        .map(|h| h.start.min(len)..h.end.min(len))
        .filter(|h| h.start < h.end)
        .collect();
    sorted.sort_by_key(|h| h.start);
    for h in sorted {
        match merged.last_mut() {
            Some(last) if h.start <= last.end => {
                if h.end > last.end {
                    last.end = h.end;
                }
            }
            _ => merged.push(h),
        }
    }

    let mut cuts: Vec<usize> = Vec::with_capacity(2 + merged.len() * 2);
    cuts.push(0);
    cuts.push(len);
    for h in &merged {
        cuts.push(h.start);
        cuts.push(h.end);
    }
    cuts.sort_unstable();
    cuts.dedup();
    cuts.windows(2)
        .filter_map(|w| {
            let (start, end) = (w[0], w[1]);
            if start >= end {
                return None;
            }
            let is_highlighted = merged
                .iter()
                .any(|h| h.start <= start && end <= h.end);
            Some(Segment {
                start,
                end,
                is_highlighted,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_line_has_no_segments() {
        assert!(segs_for_a_line(0, &[]).is_empty());
    }

    #[test]
    fn no_highlights_gives_one_whole_segment() {
        let segs = segs_for_a_line(10, &[]);
        assert_eq!(
            segs,
            vec![Segment {
                start: 0,
                end: 10,
                is_highlighted: false
            }]
        );
    }

    #[test]
    fn single_highlight_in_the_middle_gives_three_segments() {
        let segs = segs_for_a_line(10, &[3..6]);
        assert_eq!(segs.len(), 3);
        assert_eq!(segs[0], Segment { start: 0, end: 3, is_highlighted: false });
        assert_eq!(segs[1], Segment { start: 3, end: 6, is_highlighted: true });
        assert_eq!(segs[2], Segment { start: 6, end: 10, is_highlighted: false });
    }

    #[test]
    fn highlight_touching_the_start_gives_two_segments() {
        let segs = segs_for_a_line(10, &[0..4]);
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0], Segment { start: 0, end: 4, is_highlighted: true });
        assert_eq!(segs[1], Segment { start: 4, end: 10, is_highlighted: false });
    }

    #[test]
    fn highlight_touching_the_end_gives_two_segments() {
        let segs = segs_for_a_line(10, &[6..10]);
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0], Segment { start: 0, end: 6, is_highlighted: false });
        assert_eq!(segs[1], Segment { start: 6, end: 10, is_highlighted: true });
    }

    #[test]
    fn whole_line_highlight_gives_one_segment() {
        let segs = segs_for_a_line(10, &[0..10]);
        assert_eq!(
            segs,
            vec![Segment {
                start: 0,
                end: 10,
                is_highlighted: true
            }]
        );
    }

    #[test]
    fn two_disjoint_highlights_give_five_segments() {
        let segs = segs_for_a_line(20, &[2..5, 10..14]);
        assert_eq!(segs.len(), 5);
        let flags: Vec<bool> = segs.iter().map(|s| s.is_highlighted).collect();
        assert_eq!(flags, vec![false, true, false, true, false]);
    }

    #[test]
    fn overlapping_highlights_are_treated_as_one_stretch() {
        // The merge_ranges in inline_spans prevents this, but if a caller
        // passes overlapping ranges they still produce sane segments.
        let segs = segs_for_a_line(10, &[2..6, 4..8]);
        assert_eq!(segs.len(), 3);
        assert_eq!(segs[1], Segment { start: 2, end: 8, is_highlighted: true });
    }

    #[test]
    fn out_of_bounds_highlights_are_clamped() {
        let segs = segs_for_a_line(5, &[3..100]);
        // The clip turns 3..100 into 3..5.
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0], Segment { start: 0, end: 3, is_highlighted: false });
        assert_eq!(segs[1], Segment { start: 3, end: 5, is_highlighted: true });
    }

    #[test]
    fn segments_cover_the_line_with_no_gaps() {
        let segs = segs_for_a_line(20, &[3..7, 12..18]);
        assert_eq!(segs.first().map(|s| s.start), Some(0));
        assert_eq!(segs.last().map(|s| s.end), Some(20));
        for w in segs.windows(2) {
            assert_eq!(w[0].end, w[1].start, "gap or overlap");
        }
    }
}
