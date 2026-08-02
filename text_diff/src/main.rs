use codee::string::JsonSerdeCodec;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_use::storage::use_local_storage;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

#[derive(Clone)]
struct TextArea {
    getter: Signal<String>,
    setter: WriteSignal<String>,
    index: usize,
}

impl TextArea {
    fn new(index: usize, getter: Signal<String>, setter: WriteSignal<String>) -> Self {
        Self {
            getter,
            setter,
            index,
        }
    }
}

#[component]
fn App() -> impl IntoView {
    let nr_of_textareas = 5;
    let (latest_diff, latest_diff_set) = signal(String::new());

    let textareas: Vec<TextArea> = (0..nr_of_textareas)
        .map(|i| {
            let (getter, setter, _) =
                use_local_storage::<String, JsonSerdeCodec>(format!("{nr_of_textareas}{i}"));
            TextArea::new(i, getter, setter)
        })
        .collect();
    let (textareas, _) = signal(textareas);

    use leptos::html::Textarea;
    let list_of_node_refs: Vec<NodeRef<Textarea>> = (0..nr_of_textareas)
        .map(|_| NodeRef::<Textarea>::new())
        .collect();

    let (writing_status, set_writing_status) = signal(WritingStatus::Idle);

    use std::time::Duration;
    let timeout_handle: RwSignal<Option<TimeoutHandle>> = RwSignal::new(None);

    view! {
        <ForEnumerate
            each=move || textareas.get()
            key=|textarea| textarea.index
            children=move |index, textarea| {
                view! {
                    <textarea
                        prop:value=move || textarea.getter.get()
                        on:input=move |ev| {
                            let pre_keystroke_value = textarea.getter.get_untracked();
                            let new_value = event_target_value(&ev);
                            textarea.setter.set(new_value.clone());

                            let current_status = writing_status.get_untracked();

                            let old_value = match &current_status {
                                WritingStatus::Idle => pre_keystroke_value.clone(),
                                WritingStatus::Typing(snapshot) => snapshot.old_value.clone(),
                            };

                            // (2) goes here — replaces your old `should_diff_now` block
                            let is_deferred_edit = if let Some(input_ev) = ev.dyn_ref::<web_sys::InputEvent>() {
                                match input_ev.input_type().as_str() {
                                    "insertText" => input_ev
                                        .data()
                                        .map(|d| {
                                            let mut chars = d.chars();
                                            matches!((chars.next(), chars.next()), (Some(c), None) if c.is_alphabetic())
                                        })
                                        .unwrap_or(false),
                                    "deleteContentBackward" | "deleteContentForward" => {
                                        let old_len = pre_keystroke_value.chars().count();
                                        let new_len = new_value.chars().count();
                                        old_len.saturating_sub(new_len) == 1
                                    }
                                    _ => false,
                                }
                            } else {
                                false
                            };

                            if !is_deferred_edit {
                                let diff_str = format_diff(&old_value, &new_value, textarea.index);
                                latest_diff_set.set(diff_str);
                                set_writing_status.set(WritingStatus::Idle);
                                if let Some(handle) = timeout_handle.get_untracked() {
                                    handle.clear();
                                }
                                timeout_handle.set(None);
                            } else {
                                set_writing_status.set(WritingStatus::Typing(TypingSnapshot {
                                    old_value: old_value.clone(),
                                    new_value: new_value.clone(),
                                }));
                                if let Some(handle) = timeout_handle.get_untracked() {
                                    handle.clear();
                                }
                                let new_handle = set_timeout_with_handle(
                                    move || {
                                        let status = writing_status.get_untracked();
                                        if let WritingStatus::Typing(snapshot) = status {
                                            let diff_str = format_diff(&snapshot.old_value, &snapshot.new_value, textarea.index);
                                            latest_diff_set.set(diff_str);
                                        }
                                        set_writing_status.set(WritingStatus::Idle);
                                    },
                                    Duration::from_millis(1000),
                                )
                                .ok();
                                timeout_handle.set(new_handle);
                            }
                        }
                        // (3) goes here — a separate sibling attribute, same textarea element
                        on:keydown=move |ev: web_sys::KeyboardEvent| {
                            if ev.key() == "Tab" {
                                if let WritingStatus::Typing(snapshot) = writing_status.get_untracked() {
                                    let diff_str = format_diff(&snapshot.old_value, &snapshot.new_value, textarea.index);
                                    latest_diff_set.set(diff_str);
                                    set_writing_status.set(WritingStatus::Idle);
                                    if let Some(handle) = timeout_handle.get_untracked() {
                                        handle.clear();
                                    }
                                    timeout_handle.set(None);
                                }
                            }
                        }
                        node_ref=list_of_node_refs[index.get()]
                    ></textarea><br/>
                }
            }
        />

        <span>"Latest Diff"</span>
        <div id="latest-diff-result">{move || latest_diff.get()}</div>
    }
}

#[derive(Debug, Clone, PartialEq)]
struct TypingSnapshot {
    old_value: String,
    new_value: String,
}

#[derive(Debug, Clone, PartialEq)]
enum WritingStatus {
    Typing(TypingSnapshot),
    Idle,
}

/// Returns a human‑readable diff between old and new.
fn format_diff(old: &str, new: &str, box_index: usize) -> String {
    let old_chars: Vec<char> = old.chars().collect();
    let new_chars: Vec<char> = new.chars().collect();

    let first_diff =
        (0..old_chars.len().max(new_chars.len())).find(|&i| old_chars.get(i) != new_chars.get(i));

    let Some(first) = first_diff else {
        return String::new();
    };

    let old_suffix = old_chars.len() - first;
    let new_suffix = new_chars.len() - first;
    let suffix = (0..old_suffix.min(new_suffix))
        .find(|&i| old_chars[old_chars.len() - 1 - i] != new_chars[new_chars.len() - 1 - i])
        .unwrap_or(old_suffix.min(new_suffix));

    let old_end = old_chars.len() - suffix;
    let new_end = new_chars.len() - suffix;

    let old_changed: String = old_chars[first..old_end].iter().collect();
    let new_changed: String = new_chars[first..new_end].iter().collect();

    match (old_changed.is_empty(), new_changed.is_empty()) {
        (true, false) => format!("Inserted '{new_changed}' at position {first} in box {box_index}"),
        (false, true) => format!("Deleted '{old_changed}' at position {first} in box {box_index}"),
        (false, false) => format!(
            "Replaced '{old_changed}' with '{new_changed}' at position {first} in box {box_index}"
        ),
        (true, true) => String::new(),
    }
}
