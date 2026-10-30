pub mod border;
pub mod latency;
pub mod panel;
pub mod power;

use crate::client_server_coms::{ComsHandleForServer, MessageToServer};
use crate::protocol::{
    AdoptPage, DatasetGenerationPayload, GetAllPagesPayload, LoveLetter, PageRow, UnsentLetter,
    CANNED_PAGE_ID_A, CANNED_PAGE_ID_B,
};
use crate::server::border::BorderComponent;
use crate::server::latency::{
    reply_after_delay, reply_after_delay_if_still_powered, LATENCY_HEAVY_MS, LATENCY_LIGHT_MS,
};
use crate::server::panel::ServerReadout;
use crate::server::power::POWERED;
use futures::StreamExt;
use futures::channel::mpsc::UnboundedReceiver;
use futures::channel::oneshot;
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::rc::Rc;

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
            MessageToServer::AreYouOnline(tx) => self.reply_whether_online(tx, powered),
            MessageToServer::TurnOn => self.power_on(),
            MessageToServer::TurnOff => self.power_off(),
            MessageToServer::ArmFault => self.arm_next_fault(),
            other => {
                if !powered {
                    return;
                }
                self.route_while_powered(other);
            }
        }
    }

    fn route_while_powered(&self, msg: MessageToServer) {
        match msg {
            MessageToServer::PushPretendLetter { letter } => self.queue_pretend_letter(letter),
            MessageToServer::OtherClientWipes => self.simulate_other_client_wiping(),
            MessageToServer::Login { reply, .. } => self.log_client_in(reply),
            MessageToServer::GetAllPages { reply } => self.reply_with_all_pages(reply),
            MessageToServer::SendLetter { letter, reply } => {
                self.accept_letter_and_reply(letter, reply)
            }
            MessageToServer::Ack { letter_id } => self.forget_acked_letter(letter_id),
            MessageToServer::GetMyUnsentLetters { reply } => self.reply_with_unsent_letters(reply),
            MessageToServer::WipeDataset { reply } => self.wipe_dataset_and_reply(reply),
            MessageToServer::AdoptDataset { pages, reply } => {
                self.adopt_dataset_and_reply(pages, reply)
            }
            MessageToServer::AreYouOnline(_)
            | MessageToServer::TurnOn
            | MessageToServer::TurnOff
            | MessageToServer::ArmFault => {}
        }
    }

    fn reply_whether_online(&self, tx: oneshot::Sender<bool>, powered: bool) {
        reply_after_delay(LATENCY_LIGHT_MS, move || {
            let _ = tx.send(powered);
        });
    }

    fn power_on(&self) {
        POWERED.set(true);
    }

    fn power_off(&self) {
        POWERED.set(false);
        self.socket_live.set(false);
        self.handle.send_connection_lost();
    }

    fn arm_next_fault(&self) {
        if !self.fault_armed.get_untracked() {
            self.fault_armed.set(true);
        }
    }

    fn take_armed_fault(&self) -> bool {
        if self.fault_armed.get_untracked() {
            self.fault_armed.set(false);
            true
        } else {
            false
        }
    }

    fn bump_dataset_generation(&self) -> i64 {
        let new_generation = self.dataset_generation.get_untracked() + 1;
        self.dataset_generation.set(new_generation);
        new_generation
    }

    fn queue_pretend_letter(&self, letter: LoveLetter) {
        let id = self.next_letter_id.get_untracked();
        self.next_letter_id.set(id + 1);
        let unsent = UnsentLetter { letter_id: id, love_letter: letter };
        self.backlog.update(|b| b.push(unsent.clone()));
        if self.socket_live.get_untracked() {
            self.handle.push_letter(unsent);
        }
    }

    fn simulate_other_client_wiping(&self) {
        self.bump_dataset_generation();
        self.pages.set(Vec::new());
        self.backlog.set(Vec::new());
        self.socket_live.set(false);
        self.handle.send_connection_lost();
    }

    fn log_client_in(&self, reply: oneshot::Sender<Result<DatasetGenerationPayload, String>>) {
        if self.take_armed_fault() {
            reply_after_delay(LATENCY_LIGHT_MS, move || {
                let _ = reply.send(Err("login fault armed".into()));
            });
            return;
        }
        let generation = self.dataset_generation.get_untracked();
        self.socket_live.set(true);
        reply_after_delay_if_still_powered(LATENCY_LIGHT_MS, move || {
            let _ = reply.send(Ok(DatasetGenerationPayload {
                dataset_generation: generation,
            }));
        });
    }

    fn reply_with_all_pages(&self, reply: oneshot::Sender<Result<GetAllPagesPayload, String>>) {
        let pages = self.pages.get_untracked();
        reply_after_delay_if_still_powered(LATENCY_HEAVY_MS, move || {
            let _ = reply.send(Ok(GetAllPagesPayload { pages }));
        });
    }

    fn accept_letter_and_reply(
        &self,
        letter: LoveLetter,
        reply: oneshot::Sender<Result<(), String>>,
    ) {
        if self.take_armed_fault() {
            reply_after_delay(LATENCY_HEAVY_MS, move || {
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
        reply_after_delay_if_still_powered(LATENCY_HEAVY_MS, move || {
            let _ = reply.send(Ok(()));
        });
    }

    fn forget_acked_letter(&self, letter_id: i64) {
        self.backlog.update(|b| b.retain(|l| l.letter_id != letter_id));
    }

    fn reply_with_unsent_letters(
        &self,
        reply: oneshot::Sender<Result<Vec<UnsentLetter>, String>>,
    ) {
        let letters = self.backlog.get_untracked();
        reply_after_delay_if_still_powered(LATENCY_LIGHT_MS, move || {
            let _ = reply.send(Ok(letters));
        });
    }

    fn wipe_dataset_and_reply(
        &self,
        reply: oneshot::Sender<Result<DatasetGenerationPayload, String>>,
    ) {
        if self.take_armed_fault() {
            reply_after_delay(LATENCY_HEAVY_MS, move || {
                let _ = reply.send(Err("wipe fault armed".into()));
            });
            return;
        }
        let new_generation = self.bump_dataset_generation();
        self.pages.set(Vec::new());
        self.backlog.set(Vec::new());
        reply_after_delay_if_still_powered(LATENCY_HEAVY_MS, move || {
            let _ = reply.send(Ok(DatasetGenerationPayload {
                dataset_generation: new_generation,
            }));
        });
    }

    fn adopt_dataset_and_reply(
        &self,
        adopted: Vec<AdoptPage>,
        reply: oneshot::Sender<Result<DatasetGenerationPayload, String>>,
    ) {
        if self.take_armed_fault() {
            reply_after_delay(LATENCY_HEAVY_MS, move || {
                let _ = reply.send(Err("adopt fault armed".into()));
            });
            return;
        }
        let new_generation = self.bump_dataset_generation();
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
        reply_after_delay_if_still_powered(LATENCY_HEAVY_MS, move || {
            let _ = reply.send(Ok(DatasetGenerationPayload {
                dataset_generation: new_generation,
            }));
        });
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