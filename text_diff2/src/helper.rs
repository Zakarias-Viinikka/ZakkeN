use crate::diff_logic;
use crate::diff_logic::DiffResult;
use crate::text_block;
use leptos::logging::log;
use leptos::prelude::*;

pub fn update_diff(text_block: &text_block::TextBlock, new_text: String) {
    let (text, _set_text) = &text_block.text; // destructure
    let diff = diff_logic::get_diff(&text.get(), &new_text);

    match diff {
        DiffResult::Insert(text, position) => {
            log!("Insert: text={text}, position={position}");
            text_block.text.1.set(new_text)
        }
        DiffResult::Delete(text, position) => {
            log!("Delete: text={text}, position={position}");
            text_block.text.1.set(new_text)
        }
        DiffResult::NoDiff => {}
    }
    //text_block.latest_diff.set(diff);
    //text_block.latest_diff.set(text_block.diff());
}
