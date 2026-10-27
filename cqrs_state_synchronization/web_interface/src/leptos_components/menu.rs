use crate::{
    complicated_signal_flow::LocalPages,
    db,
    leptos_components::{
        action_buttons::{
            disable_page_button::DisablePageButton, edit_title_button::EditTitleButton,
        },
        dev_panel::sidebar::SidebarContainer,
        small_components::popup::create_popup,
    },
    server_link::MENU_TICK,
};
use leptos::{logging::log, prelude::*, reactive::spawn_local};
use leptos_meta::Stylesheet;
use leptos_router::components::A;

#[derive(Clone)]
struct PagesToNavTo {
    id: usize,
    title: String,
}

#[component]
pub fn Menu() -> impl IntoView {
    let (local_pages, local_pages_set) = create_local_pages();
    view! {
        <Stylesheet href="/css/main_menu.css" />

        <SidebarContainer/>

        <h1>"Menu"</h1>
        <A href="/DbGui">"ORM"</A>
        <p>"placeholder nav page"</p>

        <div id="menu_container">
            <For
                each=move || local_pages.get().into_iter().filter(|p| !p.is_disabled)
                key=|list_item| list_item.db_id.clone()
                let(list_item)
            >
                <div class="select_page_to_go_to" style="display: flex; align-items: center; gap: 8px;">
                    <EditTitleButton />
                    <A href=format!("/MainContent/{}", list_item.page_id)>
                        <button class="title_button">
                            <span>
                                {list_item.title}
                            </span>
                        </button>
                    </A>
                    <DisablePageButton
                        page_id=list_item.page_id.clone()
                        local_pages_set=local_pages_set
                    />
                </div>
            </For>
        </div>

        /*
         *
         current_title_ctr: RwSignal<usize>,
         local_pages_set: WriteSignal<Vec<LocalPages>>,
         */

    }
}

fn create_local_pages() -> (ReadSignal<Vec<LocalPages>>, WriteSignal<Vec<LocalPages>>) {
    let (local_pages, local_pages_set) = signal(Vec::<LocalPages>::new());
    let latest = RwSignal::new(0u32);

    // Refetch on mount and every time MENU_TICK bumps. The token
    // prevents a slow earlier fetch from clobbering a newer one.
    Effect::new(move |_| {
        let _ = MENU_TICK.get();
        latest.update(|n| *n = n.wrapping_add(1));
        let my_token = latest.get_untracked();
        spawn_local(async move {
            match db::db_helpers_for_web_client::get_all_menu_pages().await {
                Ok(pages) => {
                    if latest.get_untracked() == my_token {
                        local_pages_set.set(pages);
                    }
                }
                Err(e) => {
                    log!("get_all_menu_pages failed: {:?}", e);
                    create_popup(format!("get_all_menu_pages failed: {:?}", e));
                }
            }
        });
    });

    (local_pages, local_pages_set)
}
