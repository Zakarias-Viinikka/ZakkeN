use crate::client_server_coms::{ComsHandleForServer, MessageToServer};
use crate::protocol::{
    DatasetGenerationPayload, GetAllPagesPayload, LoveLetter, PageRow, UnsentLetter,
    CANNED_PAGE_ID_A, CANNED_PAGE_ID_B,
};
use futures::StreamExt;
use futures::channel::mpsc::UnboundedReceiver;
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::rc::Rc;
use std::sync::LazyLock;
use std::time::Duration;

const LATENCY_LIGHT_MS: u64 = 500;
const LATENCY_HEAVY_MS: u64 = 900;

static POWERED: LazyLock<ArcRwSignal<bool>> = LazyLock::new(|| ArcRwSignal::new(false));

pub fn powered_signal() -> ArcRwSignal<bool> {
    POWERED.clone()
}

fn delay<F: FnOnce() + 'static>(ms: u64, f: F) {
    let (tx, rx) = futures::channel::oneshot::channel::<()>();
    leptos::prelude::set_timeout(
        move || {
            let _ = tx.send(());
        },
        Duration::from_millis(ms),
    );
    spawn_local(async move {
        let _ = rx.await;
        f();
    });
}

pub struct ClientMessageReceiver {
    dataset_generation: RwSignal<i64>,
    pages: RwSignal<Vec<PageRow>>,
    backlog: RwSignal<Vec<UnsentLetter>>,
    letters_received: RwSignal<usize>,
    fault_armed: RwSignal<bool>,
    socket_live: RwSignal<bool>,
    next_letter_id: RwSignal<i64>,
    handle: ComsHandleForServer,
}

impl ClientMessageReceiver {
    fn handle(&self, msg: MessageToServer) {
        let powered = POWERED.get_untracked();
        match msg {
            MessageToServer::AreYouOnline(tx) => {
                delay(LATENCY_LIGHT_MS, move || {
                    let _ = tx.send(powered);
                });
            }
            MessageToServer::TurnOn => {
                POWERED.set(true);
            }
            MessageToServer::TurnOff => {
                POWERED.set(false);
                self.socket_live.set(false);
                self.handle.send_connection_lost();
            }
            MessageToServer::ArmFault => {
                if !self.fault_armed.get_untracked() {
                    self.fault_armed.set(true);
                }
            }
            MessageToServer::PushPretendLetter { letter } => {
                if !powered {
                    return;
                }
                let id = self.next_letter_id.get_untracked();
                self.next_letter_id.set(id + 1);
                let unsent = UnsentLetter { letter_id: id, love_letter: letter };
                self.backlog.update(|b| b.push(unsent.clone()));
                if self.socket_live.get_untracked() {
                    self.handle.push_letter(unsent);
                }
            }
            MessageToServer::OtherClientWipes => {
                if !powered {
                    return;
                }
                let new_gen = self.dataset_generation.get_untracked() + 1;
                self.dataset_generation.set(new_gen);
                self.pages.set(Vec::new());
                self.backlog.set(Vec::new());
                self.socket_live.set(false);
                self.handle.send_connection_lost();
            }
            MessageToServer::Login { reply, .. } => {
                if !powered {
                    return;
                }
                if self.fault_armed.get_untracked() {
                    self.fault_armed.set(false);
                    delay(LATENCY_LIGHT_MS, move || {
                        let _ = reply.send(Err("login fault armed".into()));
                    });
                    return;
                }
                let generation = self.dataset_generation.get_untracked();
                self.socket_live.set(true);
                let powered_sig = POWERED.clone();
                delay(LATENCY_LIGHT_MS, move || {
                    if powered_sig.get_untracked() {
                        let _ = reply.send(Ok(DatasetGenerationPayload {
                            dataset_generation: generation,
                        }));
                    }
                });
            }
            MessageToServer::GetAllPages { reply } => {
                if !powered {
                    return;
                }
                let pages = self.pages.get_untracked();
                let powered_sig = POWERED.clone();
                delay(LATENCY_HEAVY_MS, move || {
                    if powered_sig.get_untracked() {
                        let _ = reply.send(Ok(GetAllPagesPayload { pages }));
                    }
                });
            }
            MessageToServer::SendLetter { letter, reply } => {
                if !powered {
                    return;
                }
                if self.fault_armed.get_untracked() {
                    self.fault_armed.set(false);
                    delay(LATENCY_HEAVY_MS, move || {
                        let _ = reply.send(Err("send_letter fault armed".into()));
                    });
                    return;
                }
                if let LoveLetter::CreateNewPage { page_id, is_main_menu_page, .. } = &letter {
                    let already = self
                        .pages
                        .with_untracked(|p| p.iter().any(|r| &r.page_id == page_id));
                    if !already {
                        let row = PageRow {
                            page_id: page_id.clone(),
                            blobbed_page: Vec::new(),
                            page_status: Vec::new(),
                            is_main_menu_page: *is_main_menu_page,
                        };
                        self.pages.update(|p| p.push(row));
                    }
                }
                self.letters_received.update(|n| *n += 1);
                let powered_sig = POWERED.clone();
                delay(LATENCY_HEAVY_MS, move || {
                    if powered_sig.get_untracked() {
                        let _ = reply.send(Ok(()));
                    }
                });
            }
            MessageToServer::Ack { letter_id } => {
                if !powered {
                    return;
                }
                self.backlog.update(|b| b.retain(|l| l.letter_id != letter_id));
            }
            MessageToServer::GetMyUnsentLetters { reply } => {
                if !powered {
                    return;
                }
                let letters = self.backlog.get_untracked();
                let powered_sig = POWERED.clone();
                delay(LATENCY_LIGHT_MS, move || {
                    if powered_sig.get_untracked() {
                        let _ = reply.send(Ok(letters));
                    }
                });
            }
            MessageToServer::WipeDataset { reply } => {
                if !powered {
                    return;
                }
                if self.fault_armed.get_untracked() {
                    self.fault_armed.set(false);
                    delay(LATENCY_HEAVY_MS, move || {
                        let _ = reply.send(Err("wipe fault armed".into()));
                    });
                    return;
                }
                let new_generation = self.dataset_generation.get_untracked() + 1;
                self.dataset_generation.set(new_generation);
                self.pages.set(Vec::new());
                self.backlog.set(Vec::new());
                let powered_sig = POWERED.clone();
                delay(LATENCY_HEAVY_MS, move || {
                    if powered_sig.get_untracked() {
                        let _ = reply.send(Ok(DatasetGenerationPayload {
                            dataset_generation: new_generation,
                        }));
                    }
                });
            }
            MessageToServer::AdoptDataset { pages: adopted, reply } => {
                if !powered {
                    return;
                }
                if self.fault_armed.get_untracked() {
                    self.fault_armed.set(false);
                    delay(LATENCY_HEAVY_MS, move || {
                        let _ = reply.send(Err("adopt fault armed".into()));
                    });
                    return;
                }
                let new_generation = self.dataset_generation.get_untracked() + 1;
                self.dataset_generation.set(new_generation);
                let rows: Vec<PageRow> = adopted
                    .into_iter()
                    .map(|p| PageRow {
                        page_id: p.page_id,
                        blobbed_page: p.snapshot,
                        page_status: Vec::new(),
                        is_main_menu_page: p.is_main_menu_page,
                    })
                    .collect();
                self.pages.set(rows);
                self.backlog.set(Vec::new());
                let powered_sig = POWERED.clone();
                delay(LATENCY_HEAVY_MS, move || {
                    if powered_sig.get_untracked() {
                        let _ = reply.send(Ok(DatasetGenerationPayload {
                            dataset_generation: new_generation,
                        }));
                    }
                });
            }
        }
    }
}

fn seed_pages() -> Vec<PageRow> {
    vec![
        PageRow {
            page_id: CANNED_PAGE_ID_A.to_string(),
            blobbed_page: Vec::new(),
            page_status: Vec::new(),
            is_main_menu_page: true,
        },
        PageRow {
            page_id: CANNED_PAGE_ID_B.to_string(),
            blobbed_page: Vec::new(),
            page_status: Vec::new(),
            is_main_menu_page: false,
        },
    ]
}

#[derive(Clone)]
pub struct ServerReadout {
    pub powered: ArcRwSignal<bool>,
    pub dataset_generation: RwSignal<i64>,
    pub page_count: Signal<usize>,
    pub backlog_count: Signal<usize>,
    pub letters_received: RwSignal<usize>,
    pub fault_armed: RwSignal<bool>,
}

#[component]
pub fn ServerPanel() -> impl IntoView {
    let readout = expect_context::<ServerReadout>();
    let powered = readout.powered;
    let generation = readout.dataset_generation;
    let page_count = readout.page_count;
    let backlog = readout.backlog_count;
    let letters = readout.letters_received;
    let fault = readout.fault_armed;
    view! {
        <div class="side">
            <h2 class="side-title">"Server"</h2>
            <div class="side-content">
                <p>"power: " {move || if powered.get() { "on" } else { "off" }}</p>
                <p>"generation: " {move || generation.get()}</p>
                <p>"pages: " {move || page_count.get()}</p>
                <p>"unsent backlog: " {move || backlog.get()}</p>
                <p>"letters received: " {move || letters.get()}</p>
                <p>"fault armed: " {move || if fault.get() { "yes" } else { "no" }}</p>
            </div>
        </div>
    }
}

#[component]
pub fn Server(
    handle: ComsHandleForServer,
    rx: UnboundedReceiver<MessageToServer>,
    #[prop(into)] started: Signal<bool>,
    children: ChildrenFn,
) -> impl IntoView {
    let dataset_generation = RwSignal::new(1i64);
    let pages = RwSignal::new(seed_pages());
    let backlog = RwSignal::new(Vec::<UnsentLetter>::new());
    let letters_received = RwSignal::new(0usize);
    let fault_armed = RwSignal::new(false);
    let socket_live = RwSignal::new(false);
    let next_letter_id = RwSignal::new(1i64);

    let page_count = Signal::derive(move || pages.get().len());
    let backlog_count = Signal::derive(move || backlog.get().len());

    provide_context(ServerReadout {
        powered: POWERED.clone(),
        dataset_generation,
        page_count,
        backlog_count,
        letters_received,
        fault_armed,
    });

    let receiver = Rc::new(ClientMessageReceiver {
        dataset_generation,
        pages,
        backlog,
        letters_received,
        fault_armed,
        socket_live,
        next_letter_id,
        handle: handle.clone(),
    });
    let receiver_loop = receiver.clone();
    let _ = started;
    let mut rx = rx;
    spawn_local(async move {
        while let Some(msg) = rx.next().await {
            receiver_loop.handle(msg);
        }
    });

    let powered_sig = POWERED.clone();
    let border_online = Signal::derive(move || powered_sig.get());
    view! {
        <BorderComponent online=border_online>
            {children()}
        </BorderComponent>
    }
}

#[component]
fn BorderComponent(#[prop(into)] online: Signal<bool>, children: ChildrenFn) -> impl IntoView {
    let class = move || {
        if online.get() {
            "border border-online"
        } else {
            "border"
        }
    };
    view! {
        <div class=class>
            {children()}
        </div>
    }
}