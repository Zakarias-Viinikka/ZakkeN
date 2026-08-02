mod custom_text_area;
use custom_text_area::CustomTextArea;
use leptos::prelude::*;
const NUMBER_OF_TEXT_BOXES: usize = 5;
fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}
#[component]
fn App() -> impl IntoView {
    // Whichever box last had a change — describe_text_difference already includes
    // "in box N" in the message, so one shared slot is enough to know which box it was.
    let latest_diff = RwSignal::new(String::new());
    let box_indices: Vec<usize> = (0..NUMBER_OF_TEXT_BOXES).collect();
    view! {
        <div>
            <For
                each=move || box_indices.clone()
                key=|box_index| *box_index
                children=move |box_index| {
                    view! {
                        <div>
                            <CustomTextArea
                                box_index=box_index
                                on_diff=move |diff: String| {
                                    latest_diff.set(diff);
                                }
                            />
                        </div>
                    }
                }
            />
            <p>"Latest diff: " {move || latest_diff.get()}</p>
        </div>
    }
}
