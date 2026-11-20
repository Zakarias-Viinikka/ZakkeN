use crate::edit_translation::CmChange;

/// Simple deterministic PRNG so fuzz runs are 100% reproducible across native and wasm32
/// without needing `rand` / `getrandom` JS shims.
#[derive(Clone, Debug)]
pub struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0x9e37_79b9_7f4a_7c15 } else { seed },
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    pub fn gen_range(&mut self, upper_exclusive: usize) -> usize {
        if upper_exclusive <= 1 {
            return 0;
        }
        (self.next_u64() % (upper_exclusive as u64)) as usize
    }
}

const SNIPPETS: &[&str] = &[
    "",
    "a",
    "xyz ",
    "😀",
    "👨‍👩‍👧‍👦",
    "日本語",
    "café",
    "**bold**",
    "[[Cats|page-cats]]",
    "\n",
];

/// Generates 1..=3 strictly disjoint non-overlapping CM changes on valid Unicode scalar
/// boundaries (expressed in UTF-16 code units, matching CodeMirror's coordinates).
pub fn generate_random_changes(old_text: &str, rng: &mut XorShift64) -> Vec<CmChange> {
    let mut u16_bounds: Vec<u32> = Vec::with_capacity(old_text.chars().count() + 1);
    let mut u16_pos: u32 = 0;
    for ch in old_text.chars() {
        u16_bounds.push(u16_pos);
        u16_pos += ch.len_utf16() as u32;
    }
    u16_bounds.push(u16_pos);

    let char_count = u16_bounds.len() - 1;
    if char_count < 4 {
        let from_idx = rng.gen_range(u16_bounds.len());
        let to_idx = from_idx + rng.gen_range(u16_bounds.len() - from_idx);
        let insert = random_insert(from_idx == to_idx, rng);
        return vec![CmChange {
            from: u16_bounds[from_idx],
            to: u16_bounds[to_idx],
            insert,
        }];
    }

    let wanted = 1 + rng.gen_range(3);
    let mut out = Vec::with_capacity(wanted);
    let mut cursor = 0usize;
    for slot in 0..wanted {
        let remaining_slots = wanted - slot - 1;
        let max_end = char_count.saturating_sub(remaining_slots * 2);
        if cursor > max_end {
            break;
        }
        let span = max_end - cursor;
        let start = cursor + rng.gen_range((span / 2).max(1) + 1);
        let end_max = max_end.min(start + 4);
        let end = start + rng.gen_range((end_max - start) + 1);
        let insert = random_insert(start == end, rng);
        out.push(CmChange {
            from: u16_bounds[start],
            to: u16_bounds[end],
            insert,
        });
        cursor = end + 1;
    }
    out
}

fn random_insert(must_insert_nonempty: bool, rng: &mut XorShift64) -> String {
    if must_insert_nonempty {
        let idx = 1 + rng.gen_range(SNIPPETS.len() - 1);
        SNIPPETS[idx].to_string()
    } else {
        let idx = rng.gen_range(SNIPPETS.len());
        SNIPPETS[idx].to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authority_doc::AuthorityBlockDoc;
    use crate::edit_translation::{convert_changes_to_edits_desc, slice_by_utf16_range, utf16_to_byte_offset};

    fn apply_changes_to_plain_string(old: &str, changes: &[CmChange]) -> String {
        let mut out = String::new();
        let mut prev_byte = 0usize;
        for ch in changes {
            let from_b = utf16_to_byte_offset(old, ch.from).unwrap();
            let to_b = utf16_to_byte_offset(old, ch.to).unwrap();
            let _ = slice_by_utf16_range(old, ch.from, ch.to).unwrap();
            out.push_str(&old[prev_byte..from_b]);
            out.push_str(&ch.insert);
            prev_byte = to_b;
        }
        out.push_str(&old[prev_byte..]);
        out
    }

    #[test]
    fn native_fuzz_500_rounds_match_authority_doc() {
        let initial = "Start 😀 日本語 👨‍👩‍👧‍👦 café **bold** [[Cats|page-cats]]";
        let auth = AuthorityBlockDoc::new("fuzz-b1", initial);
        let mut shadow = initial.to_string();
        let mut rng = XorShift64::new(42);

        for round in 0..500 {
            let changes = generate_random_changes(&shadow, &mut rng);
            let edits = convert_changes_to_edits_desc(&shadow, &changes)
                .unwrap_or_else(|e| panic!("round {round} convert failed: {e}"));
            auth.apply_edits(&edits)
                .unwrap_or_else(|e| panic!("round {round} apply failed: {e}"));
            shadow = apply_changes_to_plain_string(&shadow, &changes);
            let actual = auth.read_text().unwrap();
            assert_eq!(actual, shadow, "mismatch at fuzz round {round}");

            if shadow.chars().count() > 240 {
                let reset = vec![CmChange {
                    from: 0,
                    to: crate::edit_translation::utf16_len(&shadow),
                    insert: initial.to_string(),
                }];
                let edits = convert_changes_to_edits_desc(&shadow, &reset).unwrap();
                auth.apply_edits(&edits).unwrap();
                shadow = initial.to_string();
            }
        }
    }
}
