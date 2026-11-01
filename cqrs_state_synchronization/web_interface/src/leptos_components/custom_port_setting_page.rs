use leptos::prelude::*;

/// Renders a marker page. Python uses this route to land the browser on a
/// URL that doesn't auto-redirect anywhere, so it can set the server port
/// before the app proceeds to setup.
#[component]
pub fn CustomPortSettingPage() -> impl IntoView {
    view! {
        <div style="text-align: center; padding: 40px; font-size: 32px;">"custom port setting page"</div>
    }
}
