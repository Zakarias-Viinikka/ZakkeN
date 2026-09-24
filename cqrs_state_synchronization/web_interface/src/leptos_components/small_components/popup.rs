use leptos::{logging::log, prelude::*};
use std::{sync::LazyLock, time::Duration};

use stylist::style; //cargo add stylist

macro_rules! for_leptos {
    ($list:expr, $item:ident => $body:expr) => {
        view! {
            <For
                each=move || $list.get()
                key=|$item| $item.id
                children=move |$item| $body
            />
        }
    };
}

static POPUPS: LazyLock<ArcRwSignal<Vec<PopupSpawner>>> =
    LazyLock::new(|| ArcRwSignal::new(Vec::new()));
static CTR: LazyLock<ArcRwSignal<i32>> = LazyLock::new(|| ArcRwSignal::new(0));

pub fn create_popup(what_to_say: String) {
    POPUPS.update(|list| list.push(PopupSpawner::new(what_to_say, CTR.get())));
    CTR.update(|c| *c += 1);
}

#[component]
pub fn PopupContainer() -> impl IntoView {
    view! {
        {for_leptos!(ArcRwSignal::clone(&POPUPS), popup => {
            view! {
                <Popup
                    begone_wordly_desire=Callback::new(move |_| remove_me(popup.id))
                    what_to_say=popup.what_to_say.clone()
                />
            }
        })}
    }
}

#[component]
pub fn Popup(begone_wordly_desire: Callback<()>, what_to_say: String) -> impl IntoView {
    let (timer, timer_s) = signal(0);
    Effect::new(move |_| {
        let handle = set_interval_with_handle(
            move || tick_the_timer(timer_s, begone_wordly_desire),
            Duration::from_millis(500),
        )
        .expect("failed to set interval");
        on_cleanup(move || handle.clear());
    });

    let styles = style!(
        background: orange;
        width: 200px;
        margin: 2px;
        color: white;
        padding: 8px;
    )
    .map_err(|e| log!("{}", e))
    .unwrap();

    view! {
        <div class=styles.get_class_name().to_string()>
            {what_to_say}
            <br/>
            {move || timer.get()}
        </div>
    }
}

fn tick_the_timer(timer_s: WriteSignal<i32>, begone_wordly_desire: Callback<()>) {
    let mut should_remove = false;
    timer_s.update(|t| {
        if *t >= 10 {
            should_remove = true;
        } else {
            *t += 1;
        }
    });
    if should_remove {
        begone_wordly_desire.run(());
    }
}

#[derive(Clone)]
pub struct PopupSpawner {
    what_to_say: String,
    pub id: i32,
}

impl PopupSpawner {
    pub fn new(what_to_say: String, id: i32) -> Self {
        Self { what_to_say, id }
    }
}

fn remove_me(id: i32) {
    ArcRwSignal::clone(&POPUPS).update(|list| list.retain(|item| item.id != id));
}
