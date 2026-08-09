use crate::app::add_css_and_js;
use crate::app::helper;
use crate::app::js_value_parsing;
use leptos::logging::log;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;
use text_diff2::custom_text_area::CustomTextArea;
use text_diff2::text_block;

use leptos_meta::*;

use crate::local_sqlite::local_sqlite_wrapper;
//use js_sys::Array;
//use wasm_bindgen::prelude::*;
//use wasm_bindgen_futures::JsFuture;

#[component]
pub fn TextBlocksPage() -> impl IntoView {
    provide_meta_context();

    let (list, set_list) = signal(Vec::new());

    let table_name = RwSignal::new("text_blocks");

    crate::javascript_take_the_wheel!("update_list_order", |js_value| {
        match js_value_parsing::js_value_to_usize_tuple(js_value) {
            Ok((old_index, new_index)) => {
                set_list.update(|v| {
                    let item = v.remove(old_index);
                    v.insert(new_index, item);
                });
                let table_name = table_name.get();
                let row_id_1 = old_index.to_string();
                let row_id_2 = new_index.to_string();
                let column = "position";
                spawn_local(async move {
                    if let Err(e) = local_sqlite_wrapper::swap_columns(
                        &table_name,
                        &row_id_1,
                        &row_id_2,
                        column,
                    )
                    .await
                    {
                        log!("javascript tried to swap two columns in the db: {:?}", e);
                    }
                });
            }
            Err(e) => log!("{}", e),
        }
    });

    Effect::new(move || {
        let table_name = table_name.get();
        spawn_local(async move {
            if let Err(e) = helper::create_table_if_not_exist().await {
                log!("create_table_if_not_exist failed: {:?}", e);
            }
            if let Err(e) = helper::create_hardcoded_columns_if_not_exist().await {
                log!("create_hardcoded_columns_if_not_exist failed: {:?}", e);
            }

            //read textblocks from local sqliteb
            let arguments = ""; //gets all
            let columns_to_read = vec!["content".to_string(), "position".to_string()];
            let localdb_data =
                local_sqlite_wrapper::get_data(table_name, arguments, &columns_to_read).await;

            match localdb_data {
                Ok(data) => {
                    for data in data.into_iter() {
                        let mut owned_strings = data.into_iter();
                        let content = owned_strings.next().unwrap();
                        let position = owned_strings.next().unwrap();
                        set_list.update(|l| {
                            l.push(text_block::TextBlock::new(
                                content,
                                RwSignal::new(position.parse().unwrap()),
                            ))
                        });
                    }
                }
                Err(e) => log!("get_data failed: {:?}", e),
            }
        })
    });

    let (last_diff, set_last_diff) = signal(String::new());

    Effect::new(move |_| {
        let diff = last_diff.get();

        if !diff.is_empty() {
            log!("Parent received diff: {}", diff);

            // Later:
            // apply_to_yjs(&diff);
            // send_to_server(&diff);
            // update_database(&diff);
        }
    });

    view! {
        <add_css_and_js::CssAndJs />
        <A href="/dbgui">"Db GUI"</A>

        <div class="finale-container">
            <ul id="sortable-container">
                <ForEnumerate
                    each=move || list.get()
                    key=|text_blocks| text_blocks.id
                    let(index, text_block)
                >
                    <TextArea
                        text_block=text_block
                        view_id=index
                    />
                </ForEnumerate>
            </ul>
        </div>
    }
}

#[component]
fn TextArea(
    #[prop(into)] text_block: text_block::TextBlock,
    #[prop(into)] view_id: ReadSignal<usize>,
) -> impl IntoView {
    view! {
        <li class="text-container" data-id={move || text_block.id.get()}>
            <div class="drag-handle">"⠿"</div>

            <div class="text-input-container">
                <CustomTextArea
                    //leptos not happy with passing async fn to child component otherwise
                    on_diff_update=move |tb, diff| {
                        spawn_local(async move {
                            if let Err(e) = helper::update_local_sqlite(tb, diff).await {
                                log!("update_local_sqlite failed: {:?}", e);
                            }
                        });
                    }
                    text_block=text_block
                    attr:class="textarea"
                    attr:id={move || view_id.get()}
                />
            </div>
        </li>
    }
}
