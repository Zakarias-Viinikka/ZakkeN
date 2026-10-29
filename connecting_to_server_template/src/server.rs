use crate::client_server_coms::{ComsHandleForServer, MessageToServer};
use crate::protocol::{
    DatasetGenerationPayload, GetAllPagesPayload, PageRow, CANNED_PAGE_ID_A, CANNED_PAGE_ID_B,
};
use futures::StreamExt;
use futures::channel::mpsc::UnboundedReceiver;
use leptos::prelude::*;
use leptos::task::spawn_local;
use std::rc::Rc;

pub struct ClientMessageReceiver {
    pub server_online: RwSignal<bool>,
    pub dataset_generation: RwSignal<i64>,
    pub pages: RwSignal<Vec<PageRow>>,
    #[allow(dead_code)]
    handle: ComsHandleForServer,
}

impl ClientMessageReceiver {
    fn handle(&self, msg: MessageToServer) {
        match msg {
            MessageToServer::AreYouOnline(tx) => {
                let _ = tx.send(self.server_online.get_untracked());
            }
            MessageToServer::Login { reply, .. } => {
                // NOTE: the real server kicks all other sockets and stamps
                // this socket with the current dataset generation. Not
                // representable here — there is only one client.
                let generation = self.dataset_generation.get_untracked();
                let _ = reply.send(Ok(DatasetGenerationPayload {
                    dataset_generation: generation,
                }));
            }
            MessageToServer::GetAllPages { reply } => {
                let pages = self.pages.get_untracked();
                let _ = reply.send(Ok(GetAllPagesPayload { pages }));
            }
            MessageToServer::SendLetter { reply, .. } => {
                // PLACEHOLDER: the real server applies the letter to its
                // pages table, stores it, and fans out unsent rows. Here it
                // logs and replies Ok.
                leptos::logging::log!("server: received SendLetter");
                let _ = reply.send(Ok(()));
            }
            MessageToServer::Ack { letter_id } => {
                leptos::logging::log!("server: ack letter_id={letter_id}");
            }
            MessageToServer::GetMyUnsentLetters { reply } => {
                // PLACEHOLDER: the real server returns the caller's unsent
                // rows. The template has no backlog table; always empty.
                let _ = reply.send(Ok(Vec::new()));
            }
            MessageToServer::WipeDataset { reply } => {
                let new_generation = self.dataset_generation.get_untracked() + 1;
                self.dataset_generation.set(new_generation);
                self.pages.set(Vec::new());
                let _ = reply.send(Ok(DatasetGenerationPayload {
                    dataset_generation: new_generation,
                }));
            }
            MessageToServer::AdoptDataset { pages: adopted, reply } => {
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
                let _ = reply.send(Ok(DatasetGenerationPayload {
                    dataset_generation: new_generation,
                }));
            }
        }
    }
}

// PLACEHOLDER: canned page rows seeded when the server starts. The real
// server reads its pages from the db.
fn seed_pages() -> Vec<PageRow> {
    vec![
        PageRow {
            page_id: CANNED_PAGE_ID_A.to_string(),
            blobbed_page: vec![],
            page_status: vec![],
            is_main_menu_page: true,
        },
        PageRow {
            page_id: CANNED_PAGE_ID_B.to_string(),
            blobbed_page: vec![],
            page_status: vec![],
            is_main_menu_page: false,
        },
    ]
}

#[derive(Clone)]
pub struct ServerReadout {
    pub dataset_generation: RwSignal<i64>,
    pub page_count: RwSignal<usize>,
}

#[component]
pub fn ServerPanel() -> impl IntoView {
    let readout = expect_context::<ServerReadout>();
    let generation = readout.dataset_generation;
    let page_count = readout.page_count;
    view! {
        <div class="side">
            <h2 class="side-title">"Server"</h2>
            <div class="side-content">
                <p>"generation: " {move || generation.get()}</p>
                <p>"pages: " {move || page_count.get()}</p>
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
    let server_online = RwSignal::new(true);
    let dataset_generation = RwSignal::new(1i64);
    let pages = RwSignal::new(seed_pages());
    let page_count_signal = RwSignal::new(pages.get_untracked().len());
    Effect::new(move |_| {
        page_count_signal.set(pages.get().len());
    });
    provide_context(ServerReadout {
        dataset_generation,
        page_count: page_count_signal,
    });
    let receiver = Rc::new(ClientMessageReceiver {
        server_online,
        dataset_generation,
        pages,
        handle: handle.clone(),
    });
    let receiver_loop = receiver.clone();
    let mut rx = rx;
    spawn_local(async move {
        while let Some(msg) = rx.next().await {
            receiver_loop.handle(msg);
        }
    });
    let border_online = Signal::derive(move || started.get() && server_online.get());
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
