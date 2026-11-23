use leptos::{html, prelude::*};
use leptos_meta::Stylesheet;

const KEYWORD_PICKER: &[(&str, MenuOption)] = &[("new page", MenuOption::MakeNewPageAndLinkToIt)];

pub enum MenuOption {
    MakeNewPageAndLinkToIt,
}

#[component]
pub fn SpecialMenuTest() -> impl IntoView {
    view!(
        <Stylesheet href="/css/special_menu_test.css"/>
        <div id="menu_container">
            <WriteField />
        </div>
    )
}

#[component]
fn WriteField() -> impl IntoView {
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
                children=|item| view! {
                    {item}<br/>
                }
        />
    )
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
