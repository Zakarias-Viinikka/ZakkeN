use crate::{
    db,
    leptos_components::{
        action_buttons::{self, page_edits::LocalPages},
        small_components::popup::PopupContainer,
    },
};
use leptos::{logging::log, prelude::*, reactive::spawn_local};
use leptos_meta::Stylesheet;
use leptos_router::components::A;
use protocol::{
    error::DbError,
    payload::{GetDataIn, JoinType, SelectArgument, SelectArguments},
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
        let arguments = SelectArguments::Two {
            first: SelectArgument::XEqualY {
                x: "is_title".to_string(),
                y: "true".to_string(),
            },
            join: JoinType::And,
            second: SelectArgument::XEqualY {
                x: "is_part_of_main_menu_page".to_string(),
                y: "true".to_string(),
            },
        };
        let get_data_in = GetDataIn {
            table_name: db::init::EVERY_BLOCK_IN_EXISTENCE_TABLE_NAME.to_string(),
            arguments,
            columns_to_read: vec!["title".to_string(), "yrs_id".to_string()],
        };
        let get_data_out = match db_helper::get_data(get_data_in).await {
            Ok(o) => o,
            Err(e) => {
                log!("get_data failed: {:?}", e);
                return;
            }
        };

        log!("{:?}", get_data_out.clone());
        local_pages_set.update(|pages| {
            for row in get_data_out.rows.into_iter() {
                let (title, yrs_id) = destruct_row_for_local_pages(row);
                pages.push(LocalPages {
                    title,
                    id: ctr.get(),
                    yrs_id,
                });
                ctr.update(|c| *c += 1);
                log!("added page to leptos' local pages list")
            }
        });
    });

    Ok((local_pages, local_pages_set))
}

fn destruct_row_for_local_pages(row: protocol::row_col::Row) -> (String, String) {
    let mut iter = row.cols.into_iter();

    let title = match iter.next() {
        Some(protocol::row_col::Col::Text(t)) => t,
        _ => panic!(),
    };

    let yrs_id = match iter.next() {
        Some(protocol::row_col::Col::Text(t)) => t,
        _ => panic!(),
    };

    (title, yrs_id)
}
