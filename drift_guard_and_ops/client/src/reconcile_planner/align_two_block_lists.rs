//! Caller pov: `align_blocks(a, b)` gives the runs of equal blocks and
//! the runs of replaced blocks between two lists of block texts.
//! `Hunk::Equal` means those blocks are identical and in place.
//! `Hunk::Replace` means a stretch on each side that has to be turned
//! into edits, removes, and creates.

pub enum Hunk {
    Equal {
        a_start: usize,
        len: usize,
    },
    Replace {
        a_start: usize,
        a_len: usize,
        b_start: usize,
        b_len: usize,
    },
}

pub fn align_blocks(a: &[String], b: &[String]) -> Vec<Hunk> {
    let matches = walk_lcs(a, b);
    let mut hunks = Vec::new();
    let (mut ai, mut bi) = (0, 0);
    for (ma, mb) in matches {
        if ma > ai || mb > bi {
            hunks.push(Hunk::Replace {
                a_start: ai,
                a_len: ma - ai,
                b_start: bi,
                b_len: mb - bi,
            });
        }
        hunks.push(Hunk::Equal {
            a_start: ma,
            len: 1,
        });
        ai = ma + 1;
        bi = mb + 1;
    }
    if ai < a.len() || bi < b.len() {
        hunks.push(Hunk::Replace {
            a_start: ai,
            a_len: a.len() - ai,
            b_start: bi,
            b_len: b.len() - bi,
        });
    }
    hunks
}

fn lcs_lengths(a: &[String], b: &[String]) -> Vec<Vec<usize>> {
    let mut dp = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            dp[i][j] = if a[i - 1] == b[j - 1] {
                dp[i - 1][j - 1] + 1
            } else {
                dp[i - 1][j].max(dp[i][j - 1])
            };
        }
    }
    dp
}

fn walk_lcs(a: &[String], b: &[String]) -> Vec<(usize, usize)> {
    let dp = lcs_lengths(a, b);
    let mut result = Vec::new();
    let (mut i, mut j) = (a.len(), b.len());
    while i > 0 && j > 0 {
        if a[i - 1] == b[j - 1] {
            result.push((i - 1, j - 1));
            i -= 1;
            j -= 1;
        } else if dp[i - 1][j] >= dp[i][j - 1] {
            i -= 1;
        } else {
            j -= 1;
        }
    }
    result.reverse();
    result
}
