use leptos::prelude::*;

#[component]
pub fn BorderComponent(#[prop(into)] online: Signal<bool>, children: ChildrenFn) -> impl IntoView {
    let class = move || {
        if online.get() {
            "border border-online"
        } else {
            "border"
        }
    };
    view! {
        <div class=class>
            {children()}
        </div>
    }
}
