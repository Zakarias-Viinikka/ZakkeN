use std::sync::{Arc, Mutex};

use futures::channel::mpsc;

use crate::text_diff_batching::happy_little_timer::{self, HappyLittleTimer, TimerInstruction};

#[derive(Debug, Clone, uniffi::Enum)]
pub enum Event {
    SingleInsert {
        character_inserted_was_space_or_newline: bool,
    }, //looks for newline, space and otherwise uses timer
    SingleDelete, //uses timer to decide when to batch
    BigInsert,    //should batch immediately
    BigDelete,    //should batch immediately
    Replace,      //should batch immediately
}

#[uniffi::export(callback_interface)]
pub trait DiffCallback: Send + Sync {
    fn on_diff_needed(&self);
}

#[derive(uniffi::Object)]
pub struct BatchTextEdits {
    talk_to_timer: mpsc::Sender<TimerInstruction>,
    batched_events: Mutex<Vec<Event>>,
    notify_caller: Arc<dyn DiffCallback>,
}

#[uniffi::export]
impl BatchTextEdits {
    #[uniffi::constructor]
    pub fn new(callback: Box<dyn DiffCallback>) -> Self {
        let callback: Arc<dyn DiffCallback> = Arc::from(callback);
        let (tx, rx) = mpsc::channel::<TimerInstruction>(10);
        let timer = Arc::new(Mutex::new(HappyLittleTimer::new(Arc::clone(&callback))));
        happy_little_timer::start(timer, rx);
        Self {
            batched_events: Mutex::new(Vec::new()),
            notify_caller: callback,
            talk_to_timer: tx,
        }
    }

    /// Feed in the next atomic edit as it happens.
    pub fn do_an_edit(&self, event: Event) {
        match &event {
            Event::SingleInsert {
                character_inserted_was_space_or_newline,
            } => {
                if *character_inserted_was_space_or_newline {
                    //TimerAction::CancelTimer
                    self.notify_caller.on_diff_needed();
                } else {
                    //TimerAction::ResetTimer
                }
            }
            Event::SingleDelete => (), //ResetTimer ,
            Event::BigInsert => self.notify_caller.on_diff_needed(),
            Event::BigDelete => self.notify_caller.on_diff_needed(),
            Event::Replace => self.notify_caller.on_diff_needed(),
        };
    }
}
