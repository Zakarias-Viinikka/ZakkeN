use crate::{
    db::{self, db_helpers_for_web_client::destruct_get_title_and_id_of_all_menu_pages},
    leptos_components::{
        action_buttons::{self},
        small_components::popup::PopupContainer,
    },
    shared_structs::LocalPages,
};
use leptos::{logging::log, prelude::*, reactive::spawn_local};
use leptos_meta::Stylesheet;
use leptos_router::components::A;
use protocol::error::DbError;

#[derive(Clone)]
struct PagesToNavTo {
    id: usize,
    title: String,
}

#[component]
pub fn Menu() -> impl IntoView {
    let ctr = RwSignal::<usize>::new(0);

    let (local_pages, local_pages_set) = create_local_pages(ctr).unwrap();
    view! {
        <Stylesheet href="/css/main_menu.css" />
        <h1>"Menu"</h1>
        <A href="/DbGui">"ORM"</A>
        <p>"placeholder nav page"</p>

        <action_buttons::PageEdits
            current_title_ctr=ctr
            local_pages=local_pages
            local_pages_set=local_pages_set
        />

        <div id="menu_container">
            <For
                each=move || local_pages.get().into_iter().filter(|p| !p.is_disabled)
                key=|list_item| list_item.id.clone()
                let(list_item)
            >
                <div class="select_page_to_go_to">
                    <button>
                        <span>
                            {list_item.title}
                        </span> <br/>
                    </button>
                </div>
            </For>
        </div>

        /*
         *
         current_title_ctr: RwSignal<usize>,
         local_pages_set: WriteSignal<Vec<LocalPages>>,
         */

        <div style="position: absolute; top: 0; right: 0;">
            <PopupContainer />
        </div>
    }
}

fn create_local_pages(
    ctr: RwSignal<usize>,
) -> Result<(ReadSignal<Vec<LocalPages>>, WriteSignal<Vec<LocalPages>>), DbError> {
    let (local_pages, local_pages_set) = signal(Vec::<LocalPages>::new());

    spawn_local(async move {
        let get_data_out =
            match db::db_helpers_for_web_client::get_title_and_id_of_all_menu_pages().await {
                Ok(o) => o,
                Err(e) => {
                    log!("get_data failed: {:?}", e);
                    return;
                }
            };

        log!("{:?}", get_data_out.clone());
        local_pages_set.update(|pages| {
            let mut rows_iter = get_data_out.rows.into_iter();
            while let Some(row) = rows_iter.next() {
                let (title, yrs_id) = destruct_get_title_and_id_of_all_menu_pages(row);
                pages.push(LocalPages {
                    title,
                    id: ctr.get_untracked(),
                    yrs_id,
                    is_disabled: false,
                });
                ctr.update(|c| *c += 1);
                log!("added page to leptos' local pages list");
            }
        });
    });

    Ok((local_pages, local_pages_set))
}
