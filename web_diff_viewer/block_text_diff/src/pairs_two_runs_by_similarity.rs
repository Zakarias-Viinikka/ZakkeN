//! Caller pov: given two slices of block text (the left slice and the
//! right slice of a hunk), return a step list. Each step is
//! (Some(old_i), Some(new_i)) for a paired row, (Some(old_i), None) for
//! a left-only step, or (None, Some(new_i)) for a right-only step, in
//! walking order. Turning steps into rows is the caller's job.
//!
//! Internals pov: weighted alignment via a score table. Diagonal steps
//! are only allowed when `pairing_similarity_if_worthy` says the pair is
//! worth pairing; the similarity itself is the gain. Deleting or
//! inserting is a step with zero gain, so the aligner prefers pairs
//! when they're available.

use crate::similarity_between_two_blocks_of_text::pairing_similarity_if_worthy;

/// Above this many cells (old × new) the DP is skipped and blocks are
/// paired by index with the same worthiness test. 100×100 is far past
/// what a real hunk reaches; anything larger falls back. Zap used
/// 100,000, tuned for short code lines. Blocks are paragraphs, so 10,000
/// is a sensible cap here.
const ALIGNMENT_CELL_CAP: usize = 10_000;

pub fn align(old: &[&str], new: &[&str]) -> Vec<(Option<usize>, Option<usize>)> {
    let k = old.len();
    let j = new.len();
    if k == 0 || j == 0 {
        let mut out = Vec::with_capacity(k + j);
        for i in 0..k {
            out.push((Some(i), None));
        }
        for i in 0..j {
            out.push((None, Some(i)));
        }
        return out;
    }
    if k * j > ALIGNMENT_CELL_CAP {
        return align_by_index(old, new);
    }

    // Precompute the worthiness matrix once. Reused on the DP pass below,
    // and reusing it here is what makes the DP O(k*j) rather than
    // O(k*j*tokens²) per cell — the similarity itself is the expensive bit.
    let mut sim: Vec<Vec<Option<f32>>> = vec![vec![None; j]; k];
    for i in 0..k {
        for jj in 0..j {
            sim[i][jj] = pairing_similarity_if_worthy(old[i], new[jj]);
        }
    }

    let mut score = vec![vec![0.0f32; j + 1]; k + 1];
    let mut trace = vec![vec![0u8; j + 1]; k + 1];

    for i in 1..=k {
        for jj in 1..=j {
            let mut best = score[i - 1][jj];
            let mut action = 0u8; // 0 = take old row alone (left-only)
            if score[i][jj - 1] > best {
                best = score[i][jj - 1];
                action = 1; // 1 = take new column alone (right-only)
            }
            if let Some(s) = sim[i - 1][jj - 1] {
                let diag = score[i - 1][jj - 1] + s;
                if diag > best {
                    best = diag;
                    action = 2; // 2 = pair them
                }
            }
            score[i][jj] = best;
            trace[i][jj] = action;
        }
    }

    let mut pairs: Vec<(Option<usize>, Option<usize>)> = Vec::with_capacity(k + j);
    let (mut i, mut jj) = (k, j);
    while i > 0 || jj > 0 {
        let action = if i == 0 {
            1
        } else if jj == 0 {
            0
        } else {
            trace[i][jj]
        };
        match action {
            0 => {
                pairs.push((Some(i - 1), None));
                i -= 1;
            }
            1 => {
                pairs.push((None, Some(jj - 1)));
                jj -= 1;
            }
            _ => {
                pairs.push((Some(i - 1), Some(jj - 1)));
                i -= 1;
                jj -= 1;
            }
        }
    }
    pairs.reverse();
    pairs
}

/// Fallback for very large slices: pair by index if the pair is
/// worthy, otherwise emit a left-only and a right-only step. Reaches the
/// same shape as the DP for aligned-by-index content, which is the case
/// where the cap would actually fire.
fn align_by_index(old: &[&str], new: &[&str]) -> Vec<(Option<usize>, Option<usize>)> {
    let mut out = Vec::new();
    let count = old.len().max(new.len());
    for i in 0..count {
        match (i < old.len(), i < new.len()) {
            (true, true) => {
                if pairing_similarity_if_worthy(old[i], new[i]).is_some() {
                    out.push((Some(i), Some(i)));
                } else {
                    out.push((Some(i), None));
                    out.push((None, Some(i)));
                }
            }
            (true, false) => out.push((Some(i), None)),
            (false, true) => out.push((None, Some(i))),
            (false, false) => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(steps: &[(Option<usize>, Option<usize>)]) -> Vec<&'static str> {
        steps
            .iter()
            .map(|&(l, r)| match (l, r) {
                (Some(_), Some(_)) => "pair",
                (Some(_), None) => "left",
                (None, Some(_)) => "right",
                (None, None) => unreachable!(),
            })
            .collect()
    }

    #[test]
    fn empty_old_gives_only_right_steps() {
        let steps = align(&[], &["a", "b"]);
        assert_eq!(kinds(&steps), vec!["right", "right"]);
        assert_eq!(steps, vec![(None, Some(0)), (None, Some(1))]);
    }

    #[test]
    fn empty_new_gives_only_left_steps() {
        let steps = align(&["a", "b"], &[]);
        assert_eq!(kinds(&steps), vec!["left", "left"]);
    }

    #[test]
    fn identical_slices_pair_everything() {
        let old = ["alpha beta gamma delta", "epsilon zeta eta theta"];
        let new = old;
        let steps = align(&old, &new);
        assert_eq!(kinds(&steps), vec!["pair", "pair"]);
        assert_eq!(steps, vec![(Some(0), Some(0)), (Some(1), Some(1))]);
    }

    #[test]
    fn dissimilar_slices_do_not_pair() {
        let old = ["yes"];
        let new = ["no"];
        let steps = align(&old, &new);
        // "yes"/"no" is not worth pairing, so both sides come out unpaired.
        assert_eq!(steps.len(), 2);
        assert_eq!(steps.iter().filter(|&&(l, _)| l.is_some()).count(), 1);
        assert_eq!(steps.iter().filter(|&&(_, r)| r.is_some()).count(), 1);
    }

    #[test]
    fn similar_short_blocks_pair() {
        let old = ["color"];
        let new = ["colour"];
        let steps = align(&old, &new);
        assert_eq!(steps, vec![(Some(0), Some(0))]);
    }

    #[test]
    fn edited_word_inside_long_block_pairs() {
        let old = ["The quick brown fox jumps over the lazy dog."];
        let new = ["The quick brown fox jumps over the lazy cat."];
        let steps = align(&old, &new);
        assert_eq!(steps, vec![(Some(0), Some(0))]);
    }

    #[test]
    fn interleaved_edit_and_insert() {
        let old = [
            "The quick brown fox jumps over the lazy dog.",
            "Then it goes to sleep for a long while.",
        ];
        let new = [
            "The quick brown fox jumps over the lazy cat.",
            "A brand new sentence with no resemblance at all.",
            "Then it goes to sleep for a long while.",
        ];
        let steps = align(&old, &new);
        // First pair is the edit. The inserted "brand new sentence"
        // is not similar to either old block, so it appears alone.
        // The third pair is the identical sleep sentence.
        assert_eq!(kinds(&steps), vec!["pair", "right", "pair"]);
        assert_eq!(steps[0], (Some(0), Some(0)));
        assert_eq!(steps[1], (None, Some(1)));
        assert_eq!(steps[2], (Some(1), Some(2)));
    }

    #[test]
    fn over_the_cap_falls_back_to_index_pairing() {
        // Deliberately over the cap. Every slot pairs by index because
        // the texts are identical.
        let text = "The quick brown fox jumps over the lazy dog many many times today";
        let old: Vec<&str> = (0..200).map(|_| text).collect();
        let new: Vec<&str> = (0..200).map(|_| text).collect();
        assert!(old.len() * new.len() > ALIGNMENT_CELL_CAP);
        let steps = align(&old, &new);
        assert_eq!(steps.len(), 200);
        for (i, step) in steps.iter().enumerate() {
            assert_eq!(*step, (Some(i), Some(i)));
        }
    }

    #[test]
    fn walk_order_is_consistent() {
        let old = ["alpha one two three", "beta one two three", "gamma one two three"];
        let new = ["alpha one two three", "gamma one two three"];
        let steps = align(&old, &new);
        // Every old index appears exactly once in order, every new index
        // appears exactly once in order.
        let old_seen: Vec<usize> = steps.iter().filter_map(|&(l, _)| l).collect();
        let new_seen: Vec<usize> = steps.iter().filter_map(|&(_, r)| r).collect();
        assert_eq!(old_seen, vec![0, 1, 2]);
        assert_eq!(new_seen, vec![0, 1]);
    }

    #[test]
    fn handles_unicode_safely() {
        let old = ["åäö text one", "another åäö line"];
        let new = ["åäö text one", "another åäö line"];
        let steps = align(&old, &new);
        assert_eq!(steps, vec![(Some(0), Some(0)), (Some(1), Some(1))]);
    }
}
