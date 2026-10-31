use leptos::prelude::*;

#[derive(Clone)]
pub struct ServerReadout {
    pub powered: ArcRwSignal<bool>,
    pub dataset_generation: RwSignal<i64>,
    pub page_count: Signal<usize>,
    pub backlog_count: Signal<usize>,
    pub letters_received: RwSignal<usize>,
    pub fault_armed: RwSignal<u32>,
}

#[component]
pub fn ServerPanel() -> impl IntoView {
    let readout = expect_context::<ServerReadout>();
    let page_count = readout.page_count;
    let letters = readout.letters_received;
    view! {
        <div class="side">
            <h2 class="side-title">"Server"</h2>
            <div class="side-content">
                <div style="display:flex; gap:3rem; justify-content:center; align-items:center;">
                    <div style="text-align:center;">
                        <div style="font-size:2.5rem; font-weight:600; line-height:1;">{move || page_count.get()}</div>
                        <div style="font-size:0.75rem; text-transform:uppercase; letter-spacing:0.12em; color:#5a6270; margin-top:0.4rem;">"pages"</div>
                    </div>
                    <div style="text-align:center;">
                        <div style="font-size:2.5rem; font-weight:600; line-height:1;">{move || letters.get()}</div>
                        <div style="font-size:0.75rem; text-transform:uppercase; letter-spacing:0.12em; color:#5a6270; margin-top:0.4rem;">"edits"</div>
                    </div>
                </div>
            </div>
        </div>
    }
}