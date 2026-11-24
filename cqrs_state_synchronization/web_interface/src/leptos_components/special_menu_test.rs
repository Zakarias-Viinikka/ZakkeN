use leptos::{html, prelude::*};
use leptos_meta::Stylesheet;

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

    let page_title_matches = create_keyword_matcher(txt_field_signal);

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

fn create_matching_titles(text_field: RwSignal<String>) -> RwSignal<Vec<String>> {
    let matching_titles = RwSignal::new(vec![]);

    Effect::new(move |_| {
        matching_titles.set(vec![]);
        matching_titles.set(
            KEYWORD_PICKER
                .iter()
                .filter(|(key, _)| key.contains(text_field.get().to_lowercase().as_str()))
                .map(|(key, _)| *key)
                .collect(),
        );
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
