use drift_guard_client::connection_status::get_conn_color;
use leptos::prelude::*;

use crate::plexget_conn_state;

#[component]
pub fn AppBorder(children: Children) -> impl IntoView {
    let color = move || get_conn_color(plexget_conn_state());
    view! {
        <div style=move || {
            format!(
                "min-height: 100vh; box-sizing: border-box; border: 4px solid {};",
                color()
            )
        }>
            <div style="padding: 1rem; padding-left: 72px;">
                {children()}
            </div>
        </div>
    }
}
