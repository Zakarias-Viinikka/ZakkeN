use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;

use crate::db::key_value_storage_db_actions::set_user_id;
use crate::leptos_components::small_components::popup::create_popup;

#[component]
pub fn SetupPage() -> impl IntoView {
    let navigate = use_navigate();
    let (name, set_name) = signal(String::new());

    let save = Callback::new(move |_: ()| {
        let name_value = name.get_untracked();
        if name_value.is_empty() {
            return;
        }
        let navigate = navigate.clone();
        spawn_local(async move {
            match set_user_id(name_value).await {
                Ok(()) => navigate("/Menu", Default::default()),
                Err(e) => {
                    leptos::logging::error!("set_user_id failed: {:?}", e);
                    create_popup(format!("save username failed: {e:?}"));
                }
            }
        });
    });

    let on_keydown = move |ev: leptos::ev::KeyboardEvent| {
        if ev.key() == "Enter" {
            save.run(());
        }
    };

    let randomize = move |_| {
        let n = (js_sys::Math::random() * 1_000_000.0) as u64;
        set_name.set(format!("user_{n}"));
    };

    view! {
        <div>
            <p>"username represents what this client calls itself when interacting with server, you can pick anything, picking an existing username has error handling so no need to worry about remembering yourself"</p>
            <input
                type="text"
                prop:value=move || name.get()
                on:input=move |ev| set_name.set(event_target_value(&ev))
                on:keydown=on_keydown
            />
            <button on:click=move |_| save.run(())>"Finish setup"</button>
            <button on:click=randomize>"Random username"</button>
        </div>
    }
}