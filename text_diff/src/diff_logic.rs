pub fn get_diff(old_text: &str, new_text: &str) -> DiffResult {
    let first_deviation = find_first_deviation(old_text, new_text);
    let last_deviation = find_last_deviation(old_text, new_text);

    if first_deviation.identical_strings {
        return DiffResult::NoDiff;
    }

    let diff_amount = last_deviation.position - first_deviation.position;

    if old_text.chars().count() == new_text.chars().count() {
        let old_middle = string_from_diff_result(old_text, first_deviation.position, diff_amount);
        let new_middle = string_from_diff_result(new_text, first_deviation.position, diff_amount);
        return DiffResult::Replace {
            old_text: old_middle,
            new_text: new_middle,
            position: first_deviation.position,
        };
    }

    if old_text.chars().count() < new_text.chars().count() {
        let diff_text = string_from_diff_result(new_text, first_deviation.position, diff_amount);
        DiffResult::Insert(diff_text, first_deviation.position)
    } else {
        let diff_text = string_from_diff_result(old_text, first_deviation.position, diff_amount);
        DiffResult::Delete(diff_text, first_deviation.position)
    }
}

fn find_first_deviation(old_text: &str, new_text: &str) -> DeviationResult {
    let mut old_text = old_text.chars();
    let mut new_text = new_text.chars();
    let mut position = 0;

    loop {
        let next_old_char = old_text.next();
        let next_new_char = new_text.next();
        if next_old_char != next_new_char {
            return DeviationResult {
                position: position,
                identical_strings: false,
            };
        } else {
            position += 1;
        }

        if next_old_char.is_none() || next_new_char.is_none() {
            return DeviationResult {
                position: 0,
                identical_strings: true,
            };
        }
    }
}

fn find_last_deviation(old_text: &str, new_text: &str) -> DeviationResult {
    let mut position = get_longest_str_len(old_text, new_text);
    let mut old_text = old_text.chars().rev();
    let mut new_text = new_text.chars().rev();

    loop {
        let next_old_char = old_text.next();
        let next_new_char = new_text.next();
        if next_old_char != next_new_char {
            return DeviationResult {
                position: position,
                identical_strings: false,
            };
        } else {
            if next_old_char.is_none() || next_new_char.is_none() {
                return DeviationResult {
                    position: 0,
                    identical_strings: true,
                };
            }

            position -= 1;
        }
    }
}

fn string_from_diff_result(old_text: &str, position_to_start_from: usize, amount: usize) -> String {
    let result: String = old_text
        .chars()
        .skip(position_to_start_from)
        .take(amount)
        .collect();

    result
}

fn get_longest_str_len(str1: &str, str2: &str) -> usize {
    let len1 = str1.chars().count();
    let len2 = str2.chars().count();
    if len1 > len2 { len1 } else { len2 }
}

struct DeviationResult {
    position: usize,
    identical_strings: bool,
}

#[derive(Debug, PartialEq)]
pub enum DiffResult {
    Insert(String, usize),
    Delete(String, usize),
    Replace {
        old_text: String,
        new_text: String,
        position: usize,
    },
    NoDiff,
}

/*
 *
 * cargo test --lib diff_logic --target x86_64-unknown-linux-gnu -- --nocapture
 *
 */
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_from_diff_result_1() {
        //let input = ("abcdef", 0, 1);
        let old_text = "abcdef";
        let position_to_start_from = 0;
        let amount = 1;
        let result = string_from_diff_result(old_text, position_to_start_from, amount);

        assert_eq!(result, "a");
    }

    #[test]
    fn test_string_from_diff_result_2() {
        //let input = ("abcdef", 0, 1);
        let old_text = "abcdef";
        let position_to_start_from = 3;
        let amount = 3;
        let result = string_from_diff_result(old_text, position_to_start_from, amount);

        assert_eq!(result, "def");
    }

    #[test]
    fn test_find_first_deviation() {
        let old_text = "fruit";
        let new_text = "fruitywuity";
        let result = find_first_deviation(old_text, new_text);
        assert_eq!(result.position, 5);
    }

    #[test]
    fn test_find_last_deviation() {
        let old_text = "fruit";
        let new_text = "fruitywuity";
        let result = find_last_deviation(old_text, new_text);
        assert_eq!(result.position, 11);
    }

    #[test]
    fn get_diff_insert() {
        let old_text = "abcdef";
        let new_text = "abcdefgh";
        let result = get_diff(old_text, new_text);
        //println!("{:?}", result);
        assert_eq!(result, DiffResult::Insert("gh".to_string(), 6));
    }

    #[test]
    fn test_get_diff_delete() {
        let old_text = "abcdefgh";
        let new_text = "abcdef";
        let result = get_diff(old_text, new_text);
        assert_eq!(result, DiffResult::Delete("gh".to_string(), 6));
    }

    #[test]
    fn test_get_diff_no_diff() {
        let old_text = "abcdef";
        let new_text = "abcdef";
        let result = get_diff(old_text, new_text);
        assert_eq!(result, DiffResult::NoDiff);
    }

    #[test]
    fn test_get_longest_str_len() {
        let str1 = "abc";
        let str2 = "defgh";
        let result = get_longest_str_len(str1, str2);
        assert_eq!(result, 5);
    }

    #[test]
    fn test_get_longest_str_len2() {
        let str1 = "defgh";
        let str2 = "blg";
        let result = get_longest_str_len(str1, str2);
        assert_eq!(result, 5);
    }

    #[test]
    fn test_replace() {
        let before = "abc";
        let after = "zzz";
        let result = get_diff(before, after);
        let expected_result = DiffResult::Replace {
            old_text: "abc".to_string(),
            new_text: "zzz".to_string(),
            position: 0,
        };
        assert_eq!(result, expected_result);
    }
}
