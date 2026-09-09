use crate::text_diff_batching::when_to_diff_timer::DiffTimer;
use std::sync::Mutex;

/// A single atomic edit, as reported by the caller in real time.
///
/// `letter` is a one-character String, not `char` — UniFFI has no
/// built-in `char` type, so `char` can't cross the FFI boundary.
#[derive(Debug, Clone, uniffi::Enum)]
pub enum WhenToDiffEvent {
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
pub struct WhenToDiff {
    timer: DiffTimer,
    text_before_latest_change: Mutex<String>,
}

#[uniffi::export]
impl WhenToDiff {
    #[uniffi::constructor]
    pub fn new(callback: Box<dyn DiffCallback>, initial_text: String) -> Self {
        let timer = DiffTimer::new(move || callback.on_diff_needed());
        Self {
            timer,
            text_before_latest_change: Mutex::new(initial_text),
        }
    }

    /// Feed in the next atomic edit as it happens.
    pub fn notify_edit(&self, event: WhenToDiffEvent) {
        let mut text = self.text_before_latest_change.lock().unwrap();

        let action = match &event {
            WhenToDiffEvent::SingleInsert { position, letter } => {
                apply_single_insert(&mut text, *position, letter)
            }
            WhenToDiffEvent::SingleDelete { position } => apply_single_delete(&mut text, *position),
            WhenToDiffEvent::BigInsert {
                position,
                text: inserted,
            } => apply_big_insert(&mut text, *position, inserted),
            WhenToDiffEvent::BigDelete { start, end } => apply_big_delete(&mut text, *start, *end),
            WhenToDiffEvent::Replace {
                start,
                end,
                text: new_text,
            } => apply_replace(&mut text, *start, *end, new_text),
        };

        drop(text); // release lock before touching the timer

        match action {
            TimerAction::Edit => self.timer.reset(),
            TimerAction::FireAndReset => self.timer.fire_and_reset(),
            TimerAction::FireAndClear => self.timer.fire_and_clear(),
        }
    }
}

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
