use leptos::prelude::*;

#[derive(Clone)]
pub struct ServerReadout {
    pub powered: ArcRwSignal<bool>,
    pub dataset_generation: RwSignal<i64>,
    pub page_count: Signal<usize>,
    pub backlog_count: Signal<usize>,
    pub letters_received: RwSignal<usize>,
    pub fault_armed: RwSignal<bool>,
}

#[component]
pub fn ServerPanel() -> impl IntoView {
    let readout = expect_context::<ServerReadout>();
    let powered = readout.powered;
    let generation = readout.dataset_generation;
    let page_count = readout.page_count;
    let backlog = readout.backlog_count;
    let letters = readout.letters_received;
    let fault = readout.fault_armed;
    view! {
        <div class="side">
            <h2 class="side-title">"Server"</h2>
            <div class="side-content">
                <p>"power: " {move || if powered.get() { "on" } else { "off" }}</p>
                <p>"generation: " {move || generation.get()}</p>
                <p>"pages: " {move || page_count.get()}</p>
                <p>"unsent backlog: " {move || backlog.get()}</p>
                <p>"letters received: " {move || letters.get()}</p>
                <p>"fault armed: " {move || if fault.get() { "yes" } else { "no" }}</p>
            </div>
        </div>
    }
}
