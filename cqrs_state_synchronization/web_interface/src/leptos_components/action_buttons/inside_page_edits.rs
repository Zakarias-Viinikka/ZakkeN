use leptos::prelude::*;
use leptos_meta::Stylesheet;

#[component]
pub fn InsidePageEdits() -> impl IntoView {
    view! {
        <Stylesheet href="/css/action_buttons_styling.css" />
        /*
        <div class="action_buttons_container">
            <span class="action_buttons_title"> "title 1"</span>
            <button>"one"</button>
            <button>"two"</button>
            <button>"three"</button>
            <button>"four"</button>

            //title 2
            <span class="action_buttons_title"> "title 2"</span>
            <button>"fee"</button>
            <button>"fai"</button>
            <button>"fee"</button>
            <button>"fooo"</button>
        </div>
         */
    }
}
