use my_yrs_lib::TextEdit;
use text_diff::diff_logic::DiffResult;

pub fn diff_result_to_text_edit(diff: DiffResult) -> Option<TextEdit> {
    match diff {
        DiffResult::Insert(text, position) => Some(TextEdit::Insert { text, position }),
        DiffResult::Delete(text, position) => Some(TextEdit::Delete { text, position }),
        DiffResult::Replace {
            old_text,
            new_text,
            position,
        } => Some(TextEdit::Replace {
            old_text,
            new_text,
            position,
        }),
        DiffResult::NoDiff => None,
    }
}
