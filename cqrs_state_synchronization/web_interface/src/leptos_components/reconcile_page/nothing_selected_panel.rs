//! Caller pov: what fills the box while no page is open. Top: the
//! default picker (Ask each time / Always mine / Always server's). Below:
//! either the "Ready to submit" prompt with a Submit button, or — after
//! Submit is clicked — a review panel listing per-page love letters and a
//! Send-and-finish action.

use leptos::prelude::*;

use block_diff_viewer_leptos::PickedSide;


#[component]
pub fn NothingSelectedPanel(
    #[prop(into)] default_picked: Signal<Option<PickedSide>>,
    on_set_default: Callback<Option<PickedSide>>,
    ready_to_submit: bool,
    review: Option<Vec<(String, Vec<String>)>>,
    review_client_only: Vec<String>,
    finishing: bool,
    on_submit_clicked: Callback<()>,
    on_review_back: Callback<()>,
    on_send_clicked: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="rp-nothing">
            <div class="rp-default-picker">
                <span class="rp-default-picker-label">"Default for everything:"</span>
                <button
                    class=move || if default_picked.get().is_none() {
                        "rp-default-btn rp-default-btn-active"
                    } else {
                        "rp-default-btn"
                    }
                    on:click=move |_| on_set_default.run(None)
                >
                    "Ask each time"
                </button>
                <button
                    class=move || if default_picked.get() == Some(PickedSide::Mine) {
                        "rp-default-btn rp-default-btn-active"
                    } else {
                        "rp-default-btn"
                    }
                    on:click=move |_| on_set_default.run(Some(PickedSide::Mine))
                >
                    "Always mine"
                </button>
                <button
                    class=move || if default_picked.get() == Some(PickedSide::Server) {
                        "rp-default-btn rp-default-btn-active"
                    } else {
                        "rp-default-btn"
                    }
                    on:click=move |_| on_set_default.run(Some(PickedSide::Server))
                >
                    "Always server's"
                </button>
            </div>
            <div class="rp-nothing-body">
                {move || {
                    if let Some(per_page) = review.clone() {
                        return view! {
                            <div class="rp-review">
                                <h2>"Review before send"</h2>
                                {if per_page.is_empty() && review_client_only.is_empty() {
                                    view! {
                                        <p>"(no love letters — every picked page matches the server)"</p>
                                    }
                                        .into_any()
                                } else {
                                    view! {
                                        <>
                                            {per_page
                                                .into_iter()
                                                .map(|(page_id, list)| {
                                                    view! {
                                                        <div class="rp-review-page">
                                                            <div class="rp-review-page-id">{page_id}</div>
                                                            {if list.is_empty() {
                                                                view! { <p>"(no letters)"</p> }.into_any()
                                                            } else {
                                                                view! {
                                                                    <ul>
                                                                        {list
                                                                            .into_iter()
                                                                            .map(|l| view! { <li>{l}</li> })
                                                                            .collect_view()}
                                                                    </ul>
                                                                }
                                                                    .into_any()
                                                            }}
                                                        </div>
                                                    }
                                                })
                                                .collect_view()}
                                            {if review_client_only.is_empty() {
                                                ().into_any()
                                            } else {
                                                view! {
                                                    <div class="rp-review-page rp-review-new">
                                                        <div class="rp-review-page-id">"New pages that will be added"</div>
                                                        <ul>
                                                            {review_client_only
                                                                .iter()
                                                                .cloned()
                                                                .map(|id| view! { <li>{id}</li> })
                                                                .collect_view()}
                                                        </ul>
                                                    </div>
                                                }
                                                    .into_any()
                                            }}
                                        </>
                                    }
                                        .into_any()
                                }}
                                <div class="rp-review-actions">
                                    <button
                                        class="rp-review-back"
                                        disabled=move || finishing
                                        on:click=move |_| on_review_back.run(())
                                    >
                                        "Back"
                                    </button>
                                    <button
                                        class="rp-submit-btn"
                                        disabled=move || finishing
                                        on:click=move |_| on_send_clicked.run(())
                                    >
                                        {if finishing { "Sending..." } else { "Send and finish" }}
                                    </button>
                                </div>
                            </div>
                        }
                            .into_any();
                    }
                    if ready_to_submit {
                        view! {
                            <div class="rp-ready">
                                <h2>"Ready to submit"</h2>
                                <p>"All picked pages will be turned into love letters."</p>
                                <button
                                    class="rp-submit-btn"
                                    disabled=move || finishing
                                    on:click=move |_| on_submit_clicked.run(())
                                >
                                    "Submit"
                                </button>
                            </div>
                        }
                            .into_any()
                    } else {
                        view! {
                            <div class="rp-ready">
                                <h2>"No diff open"</h2>
                                <p>"Click a numbered page above to open its diff."</p>
                            </div>
                        }
                            .into_any()
                    }
                }}
            </div>
        </div>
    }
}
