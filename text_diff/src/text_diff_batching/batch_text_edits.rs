use crate::diff_logic;
use crate::text_diff_batching::when_to_diff_timer::DiffTimer;
use std::sync::{Arc, Mutex};

/// A single atomic edit, as reported by the caller in real time.
///
/// `letter` is a one-character String, not `char` — UniFFI has no
/// built-in `char` type, so `char` can't cross the FFI boundary.
#[derive(Debug, Clone, uniffi::Enum)]
pub enum Event {
    SingleInsert { position: u32, letter: String },
    SingleDelete { position: u32 },
    BigInsert { position: u32, text: String },
    BigDelete { start: u32, end: u32 },
    Replace { start: u32, end: u32, text: String },
}

/// Implemented by the caller. Called whenever WhenToDiff decides
/// "now is the time to run a real text_diff and send an update".
#[uniffi::export(callback_interface)]
pub trait DiffCallback: Send + Sync {
    fn on_diff_needed(&self);
}

enum TimerAction {
    Edit,         // normal small edit, not a space -> (re)start counting down
    FireAndReset, // Big* / Replace -> fire now, keep counting down after
    FireAndClear, // space -> fire now, go back to idle (no countdown)
}

#[derive(uniffi::Object)]
pub struct BatchTextEdits {
    timer: DiffTimer,
    batched_events: Mutex<Vec<Event>>,
    previous_event: Option<Event>,
    callback_to_run_when_batch_should_be_commited:
        Arc<dyn Fn(diff_logic::DiffResult) + Send + Sync>,
}

#[uniffi::export]
impl BatchTextEdits {
    #[uniffi::constructor]
    pub fn new(callback: Box<dyn Fn(diff_logic::DiffResult) + Send + Sync>) -> Self {
        let callback = Arc::new(callback);
        let timer = DiffTimer::new(move || Arc::clone(&callback));
        Self {
            timer,
            batched_events: Mutex::new(Vec::new()),
            previous_event: None,
            callback_to_run_when_batch_should_be_commited: Arc::clone(&callback),
        }
    }

    /// Feed in the next atomic edit as it happens.
    pub fn do_an_edit(&self, event: Event) {
        let mut list_of_events = self.batched_events.lock().unwrap();

        if let Some(prev) = &self.previous_event {
            commit_old_events();
        }

        let action = match &event {
            Event::SingleInsert { position, letter } => {
                //apply_single_insert(&mut list_of_events, *position, letter)
            }
            Event::SingleDelete { position } => apply_single_delete(&mut list_of_events, *position),
            Event::BigInsert {
                position,
                text: inserted,
            } => apply_big_insert(&mut list_of_events, *position, inserted),
            Event::BigDelete { start, end } => apply_big_delete(&mut list_of_events, *start, *end),
            Event::Replace {
                start,
                end,
                text: new_text,
            } => apply_replace(&mut list_of_events, *start, *end, new_text),
        };

        drop(list_of_events); // release lock before touching the timer

        match action {
            TimerAction::Edit => self.timer.reset(),
            TimerAction::FireAndReset => self.timer.fire_and_reset(),
            TimerAction::FireAndClear => self.timer.fire_and_clear(),
        }
    }
}

fn commit_old_events(&self) {}

// --- per-event handlers: mutate the buffered text, decide the resulting action ---

fn apply_single_insert(text: &mut String, position: u32, letter: &str) -> TimerAction {
    let ch = letter.chars().next().unwrap_or_default();
    insert_char(text, position, ch);
    if letter == " " {
        TimerAction::FireAndClear
    } else {
        TimerAction::Edit
    }
}

fn apply_single_delete(text: &mut String, position: u32) -> TimerAction {
    let deleted = char_at(text, position);
    remove_char(text, position);
    if deleted == Some(' ') {
        TimerAction::FireAndClear
    } else {
        TimerAction::Edit
    }
}

fn apply_big_insert(text: &mut String, position: u32, inserted: &str) -> TimerAction {
    insert_str(text, position, inserted);
    TimerAction::FireAndReset
}

fn apply_big_delete(text: &mut String, start: u32, end: u32) -> TimerAction {
    remove_range(text, start, end);
    TimerAction::FireAndReset
}

fn apply_replace(text: &mut String, start: u32, end: u32, new_text: &str) -> TimerAction {
    remove_range(text, start, end);
    insert_str(text, start, new_text);
    TimerAction::FireAndReset
}

// --- char-index based text helpers ---

fn char_at(s: &str, index: u32) -> Option<char> {
    s.chars().nth(index as usize)
}

fn char_to_byte_index(s: &str, char_index: u32) -> usize {
    s.char_indices()
        .nth(char_index as usize)
        .map(|(byte_idx, _)| byte_idx)
        .unwrap_or(s.len())
}

fn insert_char(s: &mut String, index: u32, c: char) {
    let byte_idx = char_to_byte_index(s, index);
    s.insert(byte_idx, c);
}

fn insert_str(s: &mut String, index: u32, text: &str) {
    let byte_idx = char_to_byte_index(s, index);
    s.insert_str(byte_idx, text);
}

fn remove_char(s: &mut String, index: u32) {
    let byte_idx = char_to_byte_index(s, index);
    if byte_idx < s.len() {
        s.remove(byte_idx);
    }
}

fn remove_range(s: &mut String, start: u32, end: u32) {
    let start_byte = char_to_byte_index(s, start);
    let end_byte = char_to_byte_index(s, end);
    s.replace_range(start_byte..end_byte, "");
}
