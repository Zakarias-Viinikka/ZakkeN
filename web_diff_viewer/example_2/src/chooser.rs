//! Caller pov: the persistent row of numbered page buttons. Green when
//! picked, accent border when selected.

use leptos::prelude::*;

use crate::placeholder_data::PagePair;

const CSS: &str = r#"
.chooser {
    display: flex;
    flex-direction: column;
    gap: 10px;
    flex-shrink: 0;
}
.chooser-row {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
}
.chooser-btn {
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(228, 233, 242, 0.12);
    border-radius: 8px;
    color: rgba(228, 233, 242, 0.85);
    font-family: ui-monospace, monospace;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    transition: border-color 140ms ease, background 140ms ease, color 140ms ease;
}
.chooser-btn:hover {
    border-color: rgba(138, 180, 255, 0.4);
    background: rgba(255, 255, 255, 0.05);
}
.chooser-btn-selected {
    border-color: #8ab4ff;
    background: rgba(138, 180, 255, 0.1);
    color: #8ab4ff;
}
.chooser-btn-resolved {
    border-color: #4caf50;
    color: #6ec06e;
}
.chooser-btn-selected.chooser-btn-resolved {
    border-color: #8ab4ff;
    color: #8ab4ff;
    box-shadow: inset -3px 0 0 #4caf50;
}
"#;

#[component]
pub fn Chooser(
    #[prop(into)] pairs: Signal<Vec<PagePair>>,
    #[prop(into)] selected: Signal<Option<String>>,
    on_select: Callback<String>,
) -> impl IntoView {
    view! {
        <style>{CSS}</style>
        <div class="chooser">
            <div class="chooser-row">
                {move || {
                    pairs
                        .get()
                        .into_iter()
                        .enumerate()
                        .map(|(i, pair)| {
                            let n = i + 1;
                            let pid_for_select = pair.page_id.clone();
                            let pid_for_class = pair.page_id.clone();
                            let pid_for_resolved = pair.page_id.clone();
                            view! {
                                <button
                                    type="button"
                                    class=move || {
                                        let is_sel = selected
                                            .get()
                                            .as_deref() == Some(pid_for_class.as_str());
                                        let is_res = pairs
                                            .get()
                                            .iter()
                                            .find(|p| p.page_id == pid_for_resolved)
                                            .map(|p| p.picked.is_some())
                                            .unwrap_or(false);
                                        match (is_sel, is_res) {
                                            (true, true) => {
                                                "chooser-btn chooser-btn-selected chooser-btn-resolved"
                                            }
                                            (true, false) => "chooser-btn chooser-btn-selected",
                                            (false, true) => "chooser-btn chooser-btn-resolved",
                                            (false, false) => "chooser-btn",
                                        }
                                    }
                                    on:click=move |_| on_select.run(pid_for_select.clone())
                                >
                                    {n}
                                </button>
                            }
                        })
                        .collect_view()
                }}
            </div>
        </div>
    }
}