use super::drag_reorder_black_box::{BlockRenderInput, DragReorderBlackBox, MoveEvent, PageInput};
use super::main_for_inserting_link_to_existing_page::main_for_inserting_link_to_existing_page_at_position;
use crate::complex_state::{BlockInputEvent, BlockRow, focus};
use crate::leptos_components::special_menu::core::SpecialMenu;
use crate::leptos_components::special_menu::internally_shared::TitleMatch;
use crate::page_state::OpenPages;
use leptos::ev::{KeyboardEvent, keydown};
use leptos::prelude::*;

use super::my_block_content::MyBlockContent;
use super::shared_internally::LastEditInfo;

#[component]
pub fn DragReorderContainer(
    #[prop(into)] pages: Signal<Vec<PageInput>>,
    #[prop(into)] blocks: Signal<Vec<BlockRow>>,
    on_move: Callback<MoveEvent>,
    on_block_input: Callback<BlockInputEvent>,
    on_delete_request: Callback<String>,
    on_merge_with_above: Callback<(String, String)>,
    on_split: Callback<(String, String, String)>,
    on_navigate: Callback<(String, bool)>,
) -> impl IntoView {
    let open = expect_context::<OpenPages>();
    // Written by every block (via MyBlockContent) when its caret moves. The editors only report
    // while focused, so it stays put while the menu's input has focus.
    // Default (empty block id) means nothing was touched yet.
    let last_edit = RwSignal::new(LastEditInfo::default());
    let special_menu_is_open = RwSignal::new(false);

    let render_block = Callback::new(move |block: BlockRenderInput| {
        let on_input = Callback::new(move |evt: BlockInputEvent| {
            on_block_input.run(evt);
        });
        let on_del = Callback::new(move |bid: String| {
            on_delete_request.run(bid);
        });
        let on_merge = Callback::new(move |(bid, text): (String, String)| {
            on_merge_with_above.run((bid, text));
        });
        let on_split_cb = Callback::new(move |args: (String, String, String)| {
            on_split.run(args);
        });
        view! {
            <MyBlockContent
                block=block
                on_input=on_input
                on_delete_request=on_del
                on_merge_with_above=on_merge
                on_split=on_split_cb
                on_navigate=on_navigate
                last_edit=last_edit
            />
        }
        .into_any()
    });

    toggle_special_menu_when_user_presses_ctrl_e(special_menu_is_open, last_edit);

    view! {
        <DragReorderBlackBox
            pages=pages
            blocks=blocks
            on_move=on_move
            render_block=render_block
        />
        {move || {
            special_menu_is_open.get().then(|| view! {
                <SpecialMenu
                    cb_finish_linking_to_existing_page=move |picked: TitleMatch| {
                        let at = last_edit.get_untracked();
                        special_menu_is_open.set(false);
                        // shows a popup if the block is empty or gone, and puts the caret
                        // right after the link if it worked
                        main_for_inserting_link_to_existing_page_at_position(
                            open,
                            at.block_id,
                            at.position_last_selected,
                            picked.page_id,
                        );
                    }
                />
            })
        }}
    }
}

/// ctrl + e opens the special menu, pressing it again closes it and puts the caret back.
fn toggle_special_menu_when_user_presses_ctrl_e(
    special_menu_is_open: RwSignal<bool>,
    last_edit: RwSignal<LastEditInfo>,
) {
    let handle = window_event_listener(keydown, move |ev: KeyboardEvent| {
        if ev.ctrl_key() && ev.key() == "e" {
            //these two stop the browser's own hotkey from winning
            ev.prevent_default();
            ev.stop_propagation();
            if special_menu_is_open.get_untracked() {
                special_menu_is_open.set(false);
                let at = last_edit.get_untracked();
                if !at.block_id.is_empty() {
                    focus::at_pos(at.block_id, at.position_last_selected);
                }
            } else {
                special_menu_is_open.set(true);
            }
        }
    });
    on_cleanup(move || handle.remove());
}