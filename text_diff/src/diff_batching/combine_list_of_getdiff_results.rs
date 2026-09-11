use crate::diff_logic::{self, DiffResult};

#[uniffi::export]
pub fn combine_getdiff_results(list: Vec<diff_logic::DiffResult>) -> Vec<diff_logic::DiffResult> {
    let list = list.into_iter();
    let mut list: Vec<Option<diff_logic::DiffResult>> = list.map(Some).collect();
    let mut final_list: Vec<DiffResult> = Vec::new();
    let mut i = 0;
    while i < list.len() {
        'block: {
            let mut item = match list[i].take() {
                Some(item) => item,
                None => break 'block,
            };
            let mut ctr = 1;

            loop {
                if !item_ahead_in_bounds(i + ctr, &list) {
                    break;
                }

                let item_ahead = &list[i + ctr];

                if item_ahead.is_none() {
                    break;
                }

                let possible_combine = makes_sense_to_combine(&item, item_ahead.as_ref().unwrap());
                if possible_combine {
                    let item_ahead = list[i + ctr].take().unwrap();
                    match (item, item_ahead) {
                        (
                            DiffResult::Insert(insert_1_text, insert_1_position),
                            DiffResult::Insert(insert_2_text, _),
                        ) => {
                            let new_text = format!("{insert_1_text}{insert_2_text}");
                            item = DiffResult::Insert(new_text, insert_1_position);
                        }
                        (
                            DiffResult::Delete(delete_1_text, delete_1_position),
                            DiffResult::Delete(delete_2_text, _),
                        ) => {
                            let new_text = format!("{delete_1_text}{delete_2_text}");
                            item = DiffResult::Delete(new_text, delete_1_position);
                        }
                        _ => unreachable!(),
                    }
                    ctr = ctr + 1;
                } else {
                    break;
                }
            }
            final_list.push(item);
            i = i + ctr;
        }
    }
    final_list
}

fn item_ahead_in_bounds(i: usize, list: &[Option<diff_logic::DiffResult>]) -> bool {
    i < list.len()
}

fn makes_sense_to_combine(a: &DiffResult, b: &DiffResult) -> bool {
    match (a, b) {
        (DiffResult::Insert(a_text, a_pos), DiffResult::Insert(b_text, b_pos)) => {
            if b_text.contains(" ") || b_text.contains("\n") {
                return false;
            }
            a_pos + a_text.chars().count() as u32 == *b_pos //makes sure endposition of text a is startposition of text b
        }
        (DiffResult::Delete(a_text, a_pos), DiffResult::Delete(_, b_pos)) => {
            a_pos + a_text.chars().count() as u32 == *b_pos //makes sure endposition of text a is startposition of text b
        }
        _ => false,
    }
}

//cargo test --target x86_64-unknown-linux-gnu
#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff_logic::DiffResult;

    // --- helpers (position-aware) ---
    fn ins(s: &str, pos: u32) -> DiffResult {
        DiffResult::Insert(s.to_string(), pos)
    }
    fn del(s: &str, pos: u32) -> DiffResult {
        DiffResult::Delete(s.to_string(), pos)
    }
    fn rep(old: &str, new: &str, pos: u32) -> DiffResult {
        DiffResult::Replace {
            old_text: old.to_string(),
            new_text: new.to_string(),
            position: pos,
        }
    }

    // --- boundary ---

    #[test]
    fn empty_list_returns_empty() {
        assert_eq!(combine_getdiff_results(vec![]), vec![]);
    }

    // --- core combine ---

    #[test]
    fn three_inserts_combine() {
        assert_eq!(
            combine_getdiff_results(vec![ins("a", 0), ins("b", 1), ins("c", 2)]),
            vec![ins("abc", 0)]
        );
    }

    #[test]
    fn three_deletes_combine() {
        assert_eq!(
            combine_getdiff_results(vec![del("a", 0), del("b", 1), del("c", 2)]),
            vec![del("abc", 0)]
        );
    }

    #[test]
    fn insert_then_delete_do_not_combine() {
        assert_eq!(
            combine_getdiff_results(vec![ins("a", 0), del("b", 1)]),
            vec![ins("a", 0), del("b", 1)]
        );
    }

    // --- position adjacency ---

    #[test]
    fn non_adjacent_inserts_do_not_combine() {
        assert_eq!(
            combine_getdiff_results(vec![ins("a", 5), ins("b", 99)]),
            vec![ins("a", 5), ins("b", 99)]
        );
    }

    #[test]
    fn adjacent_inserts_combine() {
        assert_eq!(
            combine_getdiff_results(vec![ins("a", 5), ins("b", 6)]),
            vec![ins("ab", 5)]
        );
    }

    // --- whitespace rule ---

    #[test]
    fn insert_with_space_blocks_combine() {
        assert_eq!(
            combine_getdiff_results(vec![ins("a", 0), ins("b c", 1)]),
            vec![ins("a", 0), ins("b c", 1)]
        );
    }

    #[test]
    fn insert_with_newline_blocks_combine() {
        assert_eq!(
            combine_getdiff_results(vec![ins("a", 0), ins("\n", 1)]),
            vec![ins("a", 0), ins("\n", 1)]
        );
    }

    #[test]
    fn leading_space_insert_still_combines() {
        // only the incoming fragment is checked, so " " + "a" merges
        assert_eq!(
            combine_getdiff_results(vec![ins(" ", 0), ins("a", 1)]),
            vec![ins(" a", 0)]
        );
    }

    #[test]
    fn deletes_have_no_whitespace_rule() {
        assert_eq!(
            combine_getdiff_results(vec![del("a", 0), del("b c", 1)]),
            vec![del("ab c", 0)]
        );
    }

    // --- breaks and boundaries ---

    #[test]
    fn runs_separated_by_nodiff() {
        assert_eq!(
            combine_getdiff_results(vec![
                ins("a", 0),
                ins("b", 1),
                DiffResult::NoDiff,
                ins("c", 5),
                ins("d", 6),
            ]),
            vec![ins("ab", 0), DiffResult::NoDiff, ins("cd", 5)]
        );
    }

    // --- non-combining variants ---

    #[test]
    fn replaces_do_not_combine() {
        assert_eq!(
            combine_getdiff_results(vec![rep("a", "x", 0), rep("b", "y", 1)]),
            vec![rep("a", "x", 0), rep("b", "y", 1)]
        );
    }

    // --- pathological ---

    #[test]
    fn all_whitespace_inserts_never_combine() {
        assert_eq!(
            combine_getdiff_results(vec![ins(" ", 0), ins(" ", 1), ins(" ", 2)]),
            vec![ins(" ", 0), ins(" ", 1), ins(" ", 2)]
        );
    }
}
