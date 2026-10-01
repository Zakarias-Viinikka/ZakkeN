use crate::{
    db,
    leptos_components::{
        action_buttons::{self, page_edits::LocalPages},
        small_components::popup::PopupContainer,
    },
};
use client_table_blueprints::tbl_every_block_in_existence::*;
use leptos::{logging::log, prelude::*, reactive::spawn_local};
use leptos_meta::Stylesheet;
use leptos_router::components::A;
use protocol::{
    error::DbError,
    payload::{GetDataIn, JoinType, SelectArgument, SelectArguments},
    schema_helper::{DestructDbReturnCol, SchemaColumn},
};
use web_internal_db::db_helper;

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
            local_pages_set=local_pages_set
        />

        <div id="menu_container">
            <For
                each=move || local_pages.get()
                key=|list_item| list_item.title.clone()
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
                let (title, yrs_id) = destruct_row_for_local_pages(row);
                pages.push(LocalPages {
                    title,
                    id: ctr.get(),
                    yrs_id,
                });
                ctr.update(|c| *c += 1);
                log!("added page to leptos' local pages list");
            }
        });
    });

    Ok((local_pages, local_pages_set))
}

use std::collections::HashMap;

fn destruct_row_for_local_pages(row: protocol::row_col::Row) -> (String, String) {
    let schema_by_position: HashMap<u8, &SchemaColumn<String>> =
        HashMap::from([(0, &IS_TITLE), (1, &MY_ID_AS_GIVEN_BY_YRS)]);

    let mut destructed_by_name: HashMap<&str, String> = HashMap::new();

    let mut ctr: u8 = 0;
    let mut iter = row.cols.into_iter();
    while let Some(col) = iter.next() {
        let schema = schema_by_position.get(&ctr).unwrap();
        let value = schema.destruct_db_col(col).unwrap();
        destructed_by_name.insert(schema.name, value);
        ctr += 1;
    }

    let title = destructed_by_name.remove(IS_TITLE.name).unwrap();
    let yrs_id = destructed_by_name
        .remove(MY_ID_AS_GIVEN_BY_YRS.name)
        .unwrap();

    (title, yrs_id)
}
