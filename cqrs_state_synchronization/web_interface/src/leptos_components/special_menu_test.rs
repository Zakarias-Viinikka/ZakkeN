use leptos::{html, prelude::*, reactive::spawn_local};
use leptos_meta::Stylesheet;

use crate::db::reads_wrapper::search::search_page_titles;
use crate::leptos_components::small_components::popup::show_popup;

const KEYWORD_PICKER: &[(&str, MenuOption)] = &[
    ("new page", MenuOption::MakeNewPageAndLinkToIt),
    ("link to existing page", MenuOption::LinkToExistingPage),
];

#[derive(Clone)]
pub enum MenuOption {
    MakeNewPageAndLinkToIt,
    LinkToExistingPage,
}

#[component]
pub fn SpecialMenuTest() -> impl IntoView {
    let menu_state: RwSignal<Option<MenuOption>> = RwSignal::new(None);
    view!(
        <Stylesheet href="/css/special_menu_test.css"/>
        <div id="menu_container">
        {
            move || if let Some(ms) = menu_state.get() {
                match ms {
                    MenuOption::MakeNewPageAndLinkToIt => {
                        view!(<NewPage ms=menu_state />).into_any()
                    },
                    MenuOption::LinkToExistingPage => view!(<LinkToExistingPageComp ms=menu_state />).into_any()
                }
            } else {
                view!(<WriteField ms=menu_state />).into_any()
            }
        }
        </div>
    )
}

#[component]
fn NewPage(ms: RwSignal<Option<MenuOption>>) -> impl IntoView {
    view!("todo")
}

#[component]
fn LinkToExistingPageComp(ms: RwSignal<Option<MenuOption>>) -> impl IntoView {
    let txt_field_signal = RwSignal::new("".to_string());
    //focus the field at load
    let txt_field: NodeRef<html::Input> = NodeRef::new();
    Effect::new(move |_| {
        if let Some(input) = txt_field.get() {
            if let Err(err) = input.focus() {
                leptos::logging::error!("Failed to focus input: {err:?}");
            }
        }
    });

    let page_title_matches = create_matching_titles(txt_field_signal);

    view!(
            <input
                node_ref=txt_field
                type="text"
                id="special_menu_text_field"
                on:input:target=move |ev| {
                    txt_field_signal.set(ev.target().value());
                }
            />

        //show all title matches
        <For
                each=move || page_title_matches.get()
                key=|item| item.page_id.clone()
                children=move |item| {
                    let page_id = item.page_id.clone();
                    view! {
                        <button on:click=move |_| {
                            leptos::logging::log!("{}", page_id);
                        }>
                            {item.title}
                        </button><br/>
                    }
                }
        />
    )
}

#[component]
fn WriteField(ms: RwSignal<Option<MenuOption>>) -> impl IntoView {
    let txt_field_signal = RwSignal::new("".to_string());
    //focus the field at load
    let txt_field: NodeRef<html::Input> = NodeRef::new();
    Effect::new(move |_| {
        if let Some(input) = txt_field.get() {
            if let Err(err) = input.focus() {
                leptos::logging::error!("Failed to focus input: {err:?}");
            }
        }
    });

    let keyword_matches = create_keyword_matcher(txt_field_signal);

    view!(
            <input
                node_ref=txt_field
                type="text"
                id="special_menu_text_field"
                on:input:target=move |ev| {
                    txt_field_signal.set(ev.target().value());
                }
            />

        //show all keyword matches
        <For
                each=move || keyword_matches.get()
                key=|keyword| *keyword
                children=move |item| view! {
                    <button on:click=move |_| {
                        //get enum from matching string (item)
                        let e =KEYWORD_PICKER
                            .iter()
                            .find(|(key,_)| *key == item)
                            .unwrap()
                            .1
                            .clone();


                        ms.set(Some(e))
                    }>
                        {item}
                    </button><br/>
                }
        />
    )
}

#[derive(Clone)]
struct TitleMatch {
    title: String,
    page_id: String,
}

fn create_matching_titles(text_field: RwSignal<String>) -> RwSignal<Vec<TitleMatch>> {
    let matching_titles = RwSignal::new(Vec::<TitleMatch>::new());
    let latest = RwSignal::new(0u32);

    Effect::new(move |_| {
        let typed = text_field.get();
        latest.update(|n| *n = n.wrapping_add(1));
        let my_token = latest.get_untracked();



        spawn_local(async move {
            match search_page_titles(&typed).await {
                Ok(pages) => {
                    if latest.try_get_untracked() == Some(my_token) {
                        let _ = matching_titles.try_set(
                            pages
                                .into_iter()
                                .map(|p| TitleMatch {
                                    title: p.title,
                                    page_id: p.page_id,
                                })
                                .collect(),
                        );
                    }
                }
                Err(e) => {
                    leptos::logging::error!("search_page_titles failed: {:?}", e);
                    show_popup(format!("search_page_titles failed: {:?}", e));
                }
            }
        });
    });
    matching_titles
}

fn create_keyword_matcher(text_field: RwSignal<String>) -> RwSignal<Vec<&'static str>> {
    let keyword_matcher = RwSignal::new(vec![]);

    Effect::new(move |_| {
        keyword_matcher.set(vec![]);
        keyword_matcher.set(
            KEYWORD_PICKER
                .iter()
                .filter(|(key, _)| key.contains(text_field.get().to_lowercase().as_str()))
                .map(|(key, _)| *key)
                .collect(),
        );
    });
    keyword_matcher
}

//let found = ITEMS.iter().find(|&&(key, _)| key.contains(x));
