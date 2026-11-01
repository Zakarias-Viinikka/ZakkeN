//! Token-level similarity between two blocks. Shared by pairing
//! (pairs_two_runs_by_similarity.rs) and move detection
//! (moves_between_unpaired_blocks_of_text.rs). Caller pov: give me two
//! block texts and I'll tell you how similar they are, whether they're
//! worth pairing at all, and which byte ranges differ inline.

use std::ops::Range;

use similar::{Algorithm, DiffOp};

/// Long blocks (lots of tokens and lots of text) are paired at this
/// threshold. Zap's default. A false pairing is cheap on long blocks,
/// because the block is big enough that a shared row reads as a real
/// edit rather than a mis-stacked pair.
pub const PAIR_THRESHOLD_LONG: f32 = 0.5;

/// Short blocks are paired at this stricter threshold. A false pairing
/// on short blocks looks identical to a delete and an insert sharing
/// one row, which is a valid visual outcome — so the cost of getting it
/// wrong is high and the cost of not pairing is low.
pub const PAIR_THRESHOLD_SHORT: f32 = 0.7;

/// A block is "short" when it has fewer than this many tokens or fewer
/// than this many non-whitespace characters. Either side of the pair
/// being short means the pair uses the short threshold and the
/// edit-distance fallback.
pub const SHORT_BLOCK_MAX_TOKENS: usize = 4;
pub const SHORT_BLOCK_MAX_NON_WS_CHARS: usize = 24;

/// Blocks longer than this on either side skip the inline-span pass.
/// Raised from zap's 2,000 because blocks are paragraphs here, not code
/// lines.
pub const INLINE_MAX_CHARS: usize = 8_000;

/// Cheap pre-check floor for long blocks. Below this dice score the
/// LCS pass would score near zero anyway, so skip it.
pub const DICE_PREFILTER_FLOOR: f32 = 0.3;

pub fn tokenize(text: &str) -> Vec<&str> {
    let mut tokens: Vec<&str> = Vec::new();
    let mut start: Option<usize> = None;
    for (i, c) in text.char_indices() {
        let is_tok = c.is_alphanumeric() || c == '_';
        match (is_tok, start) {
            (true, None) => start = Some(i),
            (false, Some(s)) => {
                tokens.push(&text[s..i]);
                start = None;
            }
            _ => {}
        }
    }
    if let Some(s) = start {
        tokens.push(&text[s..]);
    }
    tokens
}

/// Token-level similarity. 2 * matched_weight / (total_weight_of_a +
/// total_weight_of_b), in 0..=1. Matches only full tokens, so
/// "colour"/"color" scores zero here — the edit-distance fallback in
/// `pairing_similarity_if_worthy` covers that case.
pub fn token_similarity(a: &str, b: &str) -> f32 {
    if a == b {
        return 1.0;
    }

    let ta = tokenize(a);
    let tb = tokenize(b);
    let wa: Vec<usize> = ta.iter().map(|t| t.chars().count()).collect();
    let wb: Vec<usize> = tb.iter().map(|t| t.chars().count()).collect();
    let total_a: usize = wa.iter().sum();
    let total_b: usize = wb.iter().sum();

    if total_a == 0 && total_b == 0 {
        let ac: Vec<char> = a.chars().filter(|c| !c.is_whitespace()).collect();
        let bc: Vec<char> = b.chars().filter(|c| !c.is_whitespace()).collect();
        return if ac == bc { 1.0 } else { 0.0 };
    }
    if ta.is_empty() || tb.is_empty() {
        return 0.0;
    }

    let matched = weighted_lcs(&ta, &wa, &tb);
    2.0 * matched as f32 / (total_a + total_b) as f32
}

/// Weighted longest common subsequence of the token lists, matched on
/// equality. Returns the sum of the weights of the matched tokens.
pub fn weighted_lcs(ta: &[&str], wa: &[usize], tb: &[&str]) -> usize {
    let k = ta.len();
    let j = tb.len();
    let mut dp = vec![vec![0usize; j + 1]; k + 1];
    for i in 1..=k {
        for jj in 1..=j {
            let mut best = dp[i - 1][jj].max(dp[i][jj - 1]);
            if ta[i - 1] == tb[jj - 1] {
                best = best.max(dp[i - 1][jj - 1] + wa[i - 1]);
            }
            dp[i][jj] = best;
        }
    }
    dp[k][j]
}

/// Edit distance between two strings, counted in characters. Used by
/// the short-block fallback below: "color"/"colour" scores zero on
/// token LCS but is one edit apart.
pub fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.is_empty() {
        return b.len();
    }
    if b.is_empty() {
        return a.len();
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut cur: Vec<usize> = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            let sub = prev[j] + usize::from(ca != cb);
            let best = sub.min(prev[j + 1] + 1).min(cur[j] + 1);
            cur.push(best);
        }
        prev = cur;
    }
    prev[b.len()]
}

/// Dice score over the multisets of tokens. Used as a cheap pre-check
/// before running the LCS on long blocks.
pub fn dice_over_tokens(a: &str, b: &str) -> f32 {
    let ta = tokenize(a);
    let tb = tokenize(b);
    if ta.is_empty() && tb.is_empty() {
        return 0.0;
    }
    let mut used = vec![false; tb.len()];
    let mut common = 0usize;
    for t in &ta {
        for (i, u) in tb.iter().enumerate() {
            if !used[i] && *t == *u {
                used[i] = true;
                common += 1;
                break;
            }
        }
    }
    let total = ta.len() + tb.len();
    if total == 0 {
        return 0.0;
    }
    2.0 * common as f32 / total as f32
}

fn non_ws_char_count(text: &str) -> usize {
    text.chars().filter(|c| !c.is_whitespace()).count()
}

fn is_short_pair(a: &str, b: &str) -> bool {
    let ta = tokenize(a);
    let tb = tokenize(b);
    ta.len() < SHORT_BLOCK_MAX_TOKENS
        || tb.len() < SHORT_BLOCK_MAX_TOKENS
        || non_ws_char_count(a) < SHORT_BLOCK_MAX_NON_WS_CHARS
        || non_ws_char_count(b) < SHORT_BLOCK_MAX_NON_WS_CHARS
}

/// Score if the pair is worth pairing, None otherwise. Short pairs use
/// the edit-distance fallback and the stricter threshold; long pairs
/// use the token score and the looser one. Dice-prefiltered for long
/// pairs only — short pairs skip the prefilter because a one-word
/// substitution has zero shared tokens and would be killed by it.
pub fn pairing_similarity_if_worthy(a: &str, b: &str) -> Option<f32> {
    if a == b {
        return Some(1.0);
    }
    // Empty or whitespace-only blocks only pair with an identical one
    // (already returned above). Anything else is a real diff.
    if a.trim().is_empty() || b.trim().is_empty() {
        return None;
    }
    let (score, threshold) = if is_short_pair(a, b) {
        let token = token_similarity(a, b);
        let longer = a.chars().count().max(b.chars().count());
        let edit_score = if longer == 0 {
            1.0
        } else {
            1.0 - (edit_distance(a, b) as f32 / longer as f32)
        };
        (token.max(edit_score), PAIR_THRESHOLD_SHORT)
    } else {
        if dice_over_tokens(a, b) < DICE_PREFILTER_FLOOR {
            return None;
        }
        (token_similarity(a, b), PAIR_THRESHOLD_LONG)
    };
    if score >= threshold {
        Some(score)
    } else {
        None
    }
}

/// Cut a line into tokens: a run of letters, digits and underscore, a
/// run of whitespace, or any single other character. Unlike `tokenize`,
/// keeps punctuation so an added comma can be marked.
fn token_ranges(text: &str) -> Vec<Range<usize>> {
    let mut out: Vec<Range<usize>> = Vec::new();
    let mut run: Option<(usize, u8)> = None;
    for (i, c) in text.char_indices() {
        let class: u8 = if c.is_alphanumeric() || c == '_' {
            1
        } else if c.is_whitespace() {
            2
        } else {
            0
        };
        match run {
            Some((_, k)) if k == class && class != 0 => {}
            Some((s, _)) => {
                out.push(s..i);
                run = Some((i, class));
            }
            None => run = Some((i, class)),
        }
    }
    if let Some((s, _)) = run {
        out.push(s..text.len());
    }
    out
}

fn span_of(tokens: &[Range<usize>], index: usize, len: usize) -> Option<Range<usize>> {
    if len == 0 {
        return None;
    }
    let first = tokens.get(index)?;
    let last = tokens.get(index + len - 1)?;
    Some(first.start..last.end)
}

fn merge_ranges(mut v: Vec<Range<usize>>) -> Vec<Range<usize>> {
    v.sort_by_key(|r| r.start);
    let mut out: Vec<Range<usize>> = Vec::new();
    for r in v {
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

fn mostly_covered(text: &str, spans: &[Range<usize>]) -> bool {
    let total = text.chars().filter(|c| !c.is_whitespace()).count();
    if total == 0 {
        return false;
    }
    let covered: usize = spans
        .iter()
        .map(|r| {
            text.get(r.clone())
                .map(|t| t.chars().filter(|c| !c.is_whitespace()).count())
                .unwrap_or(0)
        })
        .sum();
    covered * 10 > total * 7
}

/// Byte ranges that differ between two blocks, one list per side. Empty
/// when either block is over `INLINE_MAX_CHARS` characters, or when the
/// marks cover more than 70% of the non-space characters of either
/// side (in which case the whole line is "changed" and highlights add
/// nothing).
pub fn inline_spans(old: &str, new: &str) -> (Vec<Range<usize>>, Vec<Range<usize>>) {
    if old == new
        || old.chars().count() > INLINE_MAX_CHARS
        || new.chars().count() > INLINE_MAX_CHARS
    {
        return (Vec::new(), Vec::new());
    }
    let ta = token_ranges(old);
    let tb = token_ranges(new);
    let sa: Vec<&str> = ta.iter().map(|r| &old[r.clone()]).collect();
    let sb: Vec<&str> = tb.iter().map(|r| &new[r.clone()]).collect();

    let mut left: Vec<Range<usize>> = Vec::new();
    let mut right: Vec<Range<usize>> = Vec::new();
    for op in similar::capture_diff_slices(Algorithm::Myers, &sa, &sb) {
        match op {
            DiffOp::Equal { .. } => {}
            DiffOp::Delete { old_index, old_len, .. } => {
                if let Some(r) = span_of(&ta, old_index, old_len) {
                    left.push(r);
                }
            }
            DiffOp::Insert { new_index, new_len, .. } => {
                if let Some(r) = span_of(&tb, new_index, new_len) {
                    right.push(r);
                }
            }
            DiffOp::Replace { old_index, old_len, new_index, new_len } => {
                let (Some(o), Some(n)) = (
                    span_of(&ta, old_index, old_len),
                    span_of(&tb, new_index, new_len),
                ) else {
                    continue;
                };
                let ot = &old[o.clone()];
                let nt = &new[n.clone()];
                let pre: usize = ot
                    .chars()
                    .zip(nt.chars())
                    .take_while(|(a, b)| a == b)
                    .map(|(a, _)| a.len_utf8())
                    .sum();
                let ot2 = &ot[pre..];
                let nt2 = &nt[pre..];
                let suf: usize = ot2
                    .chars()
                    .rev()
                    .zip(nt2.chars().rev())
                    .take_while(|(a, b)| a == b)
                    .map(|(a, _)| a.len_utf8())
                    .sum();
                let lr = (o.start + pre)..(o.end - suf);
                let rr = (n.start + pre)..(n.end - suf);
                if lr.start < lr.end {
                    left.push(lr);
                }
                if rr.start < rr.end {
                    right.push(rr);
                }
            }
        }
    }

    let left = merge_ranges(left);
    let right = merge_ranges(right);
    if mostly_covered(old, &left) || mostly_covered(new, &right) {
        return (Vec::new(), Vec::new());
    }
    (left, right)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_pairs_with_colour() {
        assert!(pairing_similarity_if_worthy("color", "colour").is_some());
    }

    #[test]
    fn yes_does_not_pair_with_no() {
        assert!(pairing_similarity_if_worthy("Yes", "No").is_none());
    }

    #[test]
    fn chapter_1_pairs_with_chapter_2() {
        assert!(pairing_similarity_if_worthy("Chapter 1", "Chapter 2").is_some());
    }

    #[test]
    fn intro_does_not_pair_with_outro() {
        assert!(pairing_similarity_if_worthy("Intro", "Outro").is_none());
    }

    #[test]
    fn identical_short_blocks_always_pair() {
        assert_eq!(pairing_similarity_if_worthy("hello", "hello"), Some(1.0));
    }

    #[test]
    fn blank_blocks_only_pair_with_blank() {
        assert!(pairing_similarity_if_worthy("", "   ").is_none());
        assert!(pairing_similarity_if_worthy("hello", "   ").is_none());
    }

    #[test]
    fn long_blocks_pair_at_the_loose_threshold() {
        let a = "The quick brown fox jumps over the lazy dog and keeps on running";
        let b = "The quick brown fox jumps over the lazy cat and keeps on running";
        assert!(pairing_similarity_if_worthy(a, b).is_some());
    }

    #[test]
    fn long_blocks_that_are_mostly_different_do_not_pair() {
        let a = "The quick brown fox jumps over the lazy dog and keeps on running";
        let b = "Radically different wording with no overlap whatsoever at all";
        assert!(pairing_similarity_if_worthy(a, b).is_none());
    }

    #[test]
    fn weighted_lcs_counts_matched_weights() {
        let ta = ["alpha", "beta", "gamma"];
        let tb = ["alpha", "delta", "gamma"];
        let wa = [5, 4, 5];
        assert_eq!(weighted_lcs(&ta, &wa, &tb), 10);
    }

    #[test]
    fn edit_distance_basics() {
        assert_eq!(edit_distance("same", "same"), 0);
        assert_eq!(edit_distance("", "abc"), 3);
        assert_eq!(edit_distance("kitten", "sitting"), 3);
        assert_eq!(edit_distance("color", "colour"), 1);
    }

    #[test]
    fn dice_is_one_for_identical_tokens() {
        assert_eq!(dice_over_tokens("a b c", "a b c"), 1.0);
    }

    #[test]
    fn dice_is_zero_for_disjoint_tokens() {
        assert_eq!(dice_over_tokens("aaa bbb", "ccc ddd"), 0.0);
    }

    #[test]
    fn spans_swapped_word() {
        let (l, r) = inline_spans("call(foo, 1)", "call(bar, 1)");
        assert_eq!(l.len(), 1);
        assert_eq!(r.len(), 1);
        assert_eq!(&"call(foo, 1)"[l[0].clone()], "foo");
        assert_eq!(&"call(bar, 1)"[r[0].clone()], "bar");
    }

    #[test]
    fn spans_one_added_character() {
        let old = "let a = compute_total(x);";
        let new = "let a = compute_totals(x);";
        let (l, r) = inline_spans(old, new);
        assert!(l.is_empty());
        assert_eq!(r.len(), 1);
        assert_eq!(&new[r[0].clone()], "s");
    }

    #[test]
    fn spans_identical_lines_give_nothing() {
        let (l, r) = inline_spans("same", "same");
        assert!(l.is_empty() && r.is_empty());
    }

    #[test]
    fn spans_mostly_rewritten_line_gives_nothing() {
        let (l, r) = inline_spans("alpha beta gamma", "one two three");
        assert!(l.is_empty() && r.is_empty());
    }

    #[test]
    fn spans_multibyte_text_stays_on_char_boundaries() {
        let old = "let \u{e5} = 1;";
        let new = "let \u{f6} = 1;";
        let (l, r) = inline_spans(old, new);
        assert!(!l.is_empty() && !r.is_empty());
        assert!(l.iter().all(|x| old.get(x.clone()).is_some()));
        assert!(r.iter().all(|x| new.get(x.clone()).is_some()));
    }
}
