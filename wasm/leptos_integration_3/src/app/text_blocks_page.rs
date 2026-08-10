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
        /*
         * logic gets a bit difficult to keep in my head so i figured i'd explain it a little.
         *
         * in a way the order of the textblocks are stored in 3 different ways.
         * js has it's own order it keeps track.
         * the rust list stores items in a specific order
         * and a rust list item.id represents the position it has stored in the db
         *
         * the js and rust list are "manually" kept in sync and the reason for having to do that is so
         * js can handle the reorder animation
         *
         * the reason the db has it's "weird" way of storing position is so i can move items by setting
         * it's new position to the value between the item above and below it. otherwise i have to move
         * every single item between it's old position and it's new position
         *
         * a detail that's worth mentioning is that before commits the reorder to it's own list,
         * at that point the rust list and the js list order (should be) are the same,
         *
         * that's why rust_list[old_index_as_give_by_js]
         *
         * let's me use js the right item from the rust list
         */
        match js_value_parsing::js_value_to_usize_tuple(js_value) {
            Ok((old_index, new_index)) => {
                let text_block = list.get()[old_index]; //have to index by old_index before the js list and rust list are out of sync

                set_list.update(|v| {
                    let item: text_block::TextBlock = v.remove(old_index);
                    v.insert(new_index, item);
                });
                // for getting text_block above and below
                let snapshot = list.get();
                let above = new_index
                    .checked_sub(1)
                    .and_then(|i| snapshot.get(i))
                    .cloned();
                let below = snapshot.get(new_index + 1).cloned();
                // for getting text_block above and below

                let table_name = table_name.get();

                spawn_local(async move {
                    let result = helper::move_row(&text_block, above, below, &table_name).await;
                    if let Err(e) = result {
                        log!("failed to move row in local db: {:?}", e);
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
            let order = "ORDER BY position";
            let columns_to_read = vec!["content".to_string(), "position".to_string()];
            let localdb_data = local_sqlite_wrapper::get_data_by_order(
                table_name,
                arguments,
                &columns_to_read,
                order,
            )
            .await;

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
