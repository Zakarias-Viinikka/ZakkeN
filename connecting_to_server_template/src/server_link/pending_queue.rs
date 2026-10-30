use leptos::prelude::*;
use std::collections::VecDeque;
use std::sync::LazyLock;
use std::sync::Mutex;

static PENDING_QUEUE: LazyLock<Mutex<VecDeque<crate::protocol::LoveLetter>>> =
    LazyLock::new(|| Mutex::new(VecDeque::new()));

static PENDING_COUNT: LazyLock<ArcRwSignal<usize>> =
    LazyLock::new(|| ArcRwSignal::new(0));

pub fn pending_count_signal() -> ArcRwSignal<usize> {
    PENDING_COUNT.clone()
}

pub fn queue_pretend_edit(
    coms: &crate::client_server_coms::ComsHandleForClient,
    page_id: impl Into<String>,
    text: impl Into<String>,
) {
    let letter = crate::protocol::LoveLetter::pretend_edit(page_id, text);
    {
        let mut q = PENDING_QUEUE.lock().expect("pending queue poisoned");
        q.push_back(letter);
        PENDING_COUNT.set(q.len());
    }
    crate::server_link::sending::kick_send(coms.clone());
}

pub fn clear_pending_letters() {
    let mut q = PENDING_QUEUE.lock().expect("pending queue poisoned");
    q.clear();
    PENDING_COUNT.set(0);
}

pub fn front_pending_letter() -> Option<crate::protocol::LoveLetter> {
    let q = PENDING_QUEUE.lock().expect("pending queue poisoned");
    q.front().cloned()
}

pub fn drop_front_pending_letter() {
    let mut q = PENDING_QUEUE.lock().expect("pending queue poisoned");
    q.pop_front();
    PENDING_COUNT.set(q.len());
}
