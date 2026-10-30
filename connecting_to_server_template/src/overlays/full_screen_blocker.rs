use leptos::prelude::*;

const FSB_CSS: &str = include_str!("../../css/full_screen_blocker.css");

#[component]
pub fn FullScreenBlocker(children: Children) -> impl IntoView {
    view! {
        <style>{FSB_CSS}</style>
        <div class="fsb-blocker">
            {children()}
        </div>
    }
}
