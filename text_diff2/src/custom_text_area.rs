use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use std::time::Duration;

/// Text as it was right before the current burst of typing, and as of the latest keystroke.
#[derive(Debug, Clone, PartialEq)]
struct EditInProgress {
    text_before_edit: String,
    text_after_edit: String,
}

/// Whether this box currently has an unreported edit in progress.
#[derive(Debug, Clone, PartialEq)]
enum TypingState {
    NotTyping,
    Typing(EditInProgress),
}

#[component]
pub fn CustomTextArea(
    /// Which box this is. Only used to label the diff so the parent knows where it came from.
    box_index: usize,
    /// Called with a description of the change whenever an edit finishes.
    #[prop(into)]
    on_diff: Callback<String>,
) -> impl IntoView {
    let current_text = RwSignal::new(String::new());
    let typing_state = RwSignal::new(TypingState::NotTyping);
    let pending_timer: RwSignal<Option<TimeoutHandle>> = RwSignal::new(None);
    let node_ref = NodeRef::<leptos::html::Textarea>::new();

    view! {
        <textarea
            prop:value=move || current_text.get()
            on:input=move |ev| {
                let text_before_this_keystroke = current_text.get_untracked();
                let text_after_this_keystroke = event_target_value(&ev);
                current_text.set(text_after_this_keystroke.clone());

                let text_before_edit = match typing_state.get_untracked() {
                    TypingState::NotTyping => text_before_this_keystroke.clone(),
                    TypingState::Typing(edit) => edit.text_before_edit,
                    _ => unreachable!("unexpected typing state"),
                };

                let wait_for_more_typing = match ev.dyn_ref::<web_sys::InputEvent>() {
                    Some(input_event) => match input_event.input_type().as_str() {
                        "insertText" => input_event
                            .data()
                            .map(|inserted| {
                                let mut chars = inserted.chars();
                                matches!((chars.next(), chars.next()), (Some(c), None) if c.is_alphabetic())
                            })
                            .unwrap_or(false),
                        "deleteContentBackward" | "deleteContentForward" => {
                            let removed = text_before_this_keystroke
                                .chars()
                                .count()
                                .saturating_sub(text_after_this_keystroke.chars().count());
                            removed == 1
                        }
                        _ => false,
                    },
                    None => false,
                };

                if wait_for_more_typing {
                    typing_state.set(TypingState::Typing(EditInProgress {
                        text_before_edit: text_before_edit.clone(),
                        text_after_edit: text_after_this_keystroke.clone(),
                    }));

                    cancel_pending_timer(pending_timer);
                    let on_diff = on_diff.clone();
                    let handle = set_timeout_with_handle(
                        move || {
                            if let TypingState::Typing(edit) = typing_state.get_untracked() {
                                report_and_reset(
                                    on_diff.clone(),
                                    typing_state,
                                    pending_timer,
                                    box_index,
                                    &edit.text_before_edit,
                                    &edit.text_after_edit,
                                );
                            }
                        },
                        Duration::from_millis(300),
                    )
                    .ok();
                    pending_timer.set(handle);
                } else {
                    report_and_reset(
                        on_diff.clone(),
                        typing_state,
                        pending_timer,
                        box_index,
                        &text_before_edit,
                        &text_after_this_keystroke,
                    );
                }
            }

            on:keydown=move |ev: web_sys::KeyboardEvent| {
                // Tab never fires an input event, so it needs its own flush.
                if ev.key() == "Tab" {
                    if let TypingState::Typing(edit) = typing_state.get_untracked() {
                        report_and_reset(
                            on_diff.clone(),
                            typing_state,
                            pending_timer,
                            box_index,
                            &edit.text_before_edit,
                            &edit.text_after_edit,
                        );
                    }
                }
            }

            node_ref=node_ref
        ></textarea>
    }
}

/// Cancels whatever "flush the diff soon" timer is currently pending, if any.
fn cancel_pending_timer(pending_timer: RwSignal<Option<TimeoutHandle>>) {
    if let Some(handle) = pending_timer.get_untracked() {
        handle.clear();
    }
    pending_timer.set(None);
}

/// Sends the diff to the parent, then resets this box back to "not typing".
fn report_and_reset(
    on_diff: Callback<String>,
    typing_state: RwSignal<TypingState>,
    pending_timer: RwSignal<Option<TimeoutHandle>>,
    box_index: usize,
    text_before: &str,
    text_after: &str,
) {
    on_diff.run(describe_text_difference(text_before, text_after, box_index));
    typing_state.set(TypingState::NotTyping);
    cancel_pending_timer(pending_timer);
}

/// Describes what changed between two versions of a box's text, e.g.
/// "Inserted 'x' at position 4 in box 2".
fn describe_text_difference(text_before: &str, text_after: &str, box_index: usize) -> String {
    let before: Vec<char> = text_before.chars().collect();
    let after: Vec<char> = text_after.chars().collect();

    let Some(first_diff) =
        (0..before.len().max(after.len())).find(|&i| before.get(i) != after.get(i))
    else {
        return String::new();
    };

    let before_remaining = before.len() - first_diff;
    let after_remaining = after.len() - first_diff;
    let matching_suffix_len = (0..before_remaining.min(after_remaining))
        .find(|&i| before[before.len() - 1 - i] != after[after.len() - 1 - i])
        .unwrap_or(before_remaining.min(after_remaining));

    let before_end = before.len() - matching_suffix_len;
    let after_end = after.len() - matching_suffix_len;

    let removed_text: String = before[first_diff..before_end].iter().collect();
    let added_text: String = after[first_diff..after_end].iter().collect();

    match (removed_text.is_empty(), added_text.is_empty()) {
        (true, false) => {
            format!("Inserted '{added_text}' at position {first_diff} in box {box_index}")
        }
        (false, true) => {
            format!("Deleted '{removed_text}' at position {first_diff} in box {box_index}")
        }
        (false, false) => format!(
            "Replaced '{removed_text}' with '{added_text}' at position {first_diff} in box {box_index}"
        ),
        (true, true) => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::describe_text_difference;

    #[test]
    fn identical_strings_return_empty() {
        assert_eq!(describe_text_difference("hello", "hello", 0), "");
    }

    #[test]
    fn insertion_at_start() {
        assert_eq!(
            describe_text_difference("world", "hello world", 1),
            "Inserted 'hello ' at position 0 in box 1"
        );
    }

    #[test]
    fn insertion_in_middle() {
        assert_eq!(
            describe_text_difference("abcz", "abcdefgz", 2),
            "Inserted 'defg' at position 3 in box 2"
        );
    }

    #[test]
    fn insertion_at_end() {
        assert_eq!(
            describe_text_difference("start", "start end", 3),
            "Inserted ' end' at position 5 in box 3"
        );
    }

    #[test]
    fn deletion_at_start() {
        assert_eq!(
            describe_text_difference("prefix text", "text", 0),
            "Deleted 'prefix ' at position 0 in box 0"
        );
    }

    #[test]
    fn deletion_in_middle() {
        assert_eq!(
            describe_text_difference("abcdefg", "abg", 1),
            "Deleted 'cdef' at position 2 in box 1"
        );
    }

    #[test]
    fn deletion_at_end() {
        assert_eq!(
            describe_text_difference("data goes here", "data", 2),
            "Deleted ' goes here' at position 4 in box 2"
        );
    }

    #[test]
    fn replacement_simple() {
        assert_eq!(
            describe_text_difference("old", "new", 3),
            "Replaced 'old' with 'new' at position 0 in box 3"
        );
    }

    #[test]
    fn replacement_with_common_prefix_and_suffix() {
        assert_eq!(
            describe_text_difference("abc---xyz", "abc+++xyz", 4),
            "Replaced '---' with '+++' at position 3 in box 4"
        );
    }

    #[test]
    fn unicode_characters() {
        assert_eq!(
            describe_text_difference("café", "café au lait", 5),
            "Inserted ' au lait' at position 4 in box 5"
        );
    }

    #[test]
    fn emoji_insertion() {
        assert_eq!(
            describe_text_difference("start 🎉", "start 😊🎉", 6),
            "Inserted '😊' at position 6 in box 6"
        );
    }

    #[test]
    fn multi_byte_removal() {
        assert_eq!(
            describe_text_difference("こんにちは世界", "こんにちは", 7),
            "Deleted '世界' at position 5 in box 7"
        );
    }

    #[test]
    fn overlapping_common_part_inside() {
        // When the changed section itself contains parts that match prefix/suffix,
        // the algorithm still works because it finds the *first* differing character
        // and the *last* differing character.
        assert_eq!(
            describe_text_difference("axbxc", "aybyc", 0),
            "Replaced 'xbx' with 'yby' at position 1 in box 0"
        );
    }

    #[test]
    fn whole_string_deleted() {
        assert_eq!(
            describe_text_difference("everything", "", 3),
            "Deleted 'everything' at position 0 in box 3"
        );
    }

    #[test]
    fn whole_string_inserted() {
        assert_eq!(
            describe_text_difference("", "everything", 1),
            "Inserted 'everything' at position 0 in box 1"
        );
    }
}
