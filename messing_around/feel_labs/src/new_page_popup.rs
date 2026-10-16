use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use std::time::Duration;
use wasm_bindgen::JsCast;

const SHAKE_AMPLITUDE_PX: i32 = 6;
const SHAKE_DURATION_MS: u32 = 320;
const OVERLAY_OPACITY: f64 = 0.45;
const PANEL_TRANSITION_MS: u32 = 220;

#[derive(Clone)]
struct Letter {
    id: u64,
    c: char,
}

fn apply_edit(old: &[Letter], new_str: &str, next_id: &mut u64) -> Vec<Letter> {
    let new_chars: Vec<char> = new_str.chars().collect();
    let mut p = 0usize;
    while p < old.len() && p < new_chars.len() && old[p].c == new_chars[p] {
        p += 1;
    }
    let mut s = 0usize;
    while s < old.len().saturating_sub(p)
        && s < new_chars.len().saturating_sub(p)
        && old[old.len() - 1 - s].c == new_chars[new_chars.len() - 1 - s]
    {
        s += 1;
    }

    let mut result: Vec<Letter> = Vec::with_capacity(new_chars.len());
    for l in old.iter().take(p) {
        result.push(l.clone());
    }
    for c in new_chars.iter().take(new_chars.len() - s).skip(p) {
        result.push(Letter { id: *next_id, c: *c });
        *next_id = next_id.wrapping_add(1);
    }
    if s > 0 {
        for l in old.iter().skip(old.len() - s) {
            result.push(l.clone());
        }
    }
    result
}

#[component]
pub fn NewPagePopup() -> impl IntoView {
    let root_style = format!(
        "--npp-overlay: {overlay}; --npp-panel-dur: {panel}ms; \
         --npp-shake-amp: {shake_amp}px; --npp-shake-dur: {shake_ms}ms;",
        overlay = OVERLAY_OPACITY,
        panel = PANEL_TRANSITION_MS,
        shake_amp = SHAKE_AMPLITUDE_PX,
        shake_ms = SHAKE_DURATION_MS,
    );

    let popup_open = RwSignal::new(true);
    let letters = RwSignal::new(Vec::<Letter>::new());
    let next_id = RwSignal::new(1u64);
    let result = RwSignal::new(None::<String>);

    let input_ref = NodeRef::<html::Input>::new();

    Effect::new(move |_| {
        if popup_open.get() {
            set_timeout(
                move || {
                    if let Some(el) = input_ref.get() {
                        if let Err(e) = el.focus() {
                            leptos::logging::error!(
                                "new_page_popup: focus input: {e:?}"
                            );
                        }
                    }
                },
                Duration::from_millis(0),
            );
        }
    });

    let clear_input = move || {
        letters.set(Vec::new());
        if let Some(el) = input_ref.get_untracked() {
            el.set_value("");
        }
    };

    let close_cancel = move || {
        popup_open.set(false);
        clear_input();
    };

    let close_confirm = move || {
        let title: String = letters
            .with_untracked(|list| list.iter().map(|l| l.c).collect());
        let trimmed = title.trim();
        result.set(if trimmed.is_empty() { None } else { Some(trimmed.to_string()) });
        popup_open.set(false);
        clear_input();
    };

    let on_input = move |ev: ev::Event| {
        let Some(target) = ev.target() else {
            leptos::logging::error!("new_page_popup: input event has no target");
            return;
        };
        let Ok(inp) = target.dyn_into::<web_sys::HtmlInputElement>() else {
            leptos::logging::error!("new_page_popup: input target is not an input");
            return;
        };
        let new_str = inp.value();
        let mut nid = next_id.get_untracked();
        let updated = letters.with_untracked(|old| apply_edit(old, &new_str, &mut nid));
        next_id.set(nid);
        letters.set(updated);
    };

    let on_keydown = move |ev: ev::KeyboardEvent| {
        match ev.key().as_str() {
            "Enter" => {
                ev.prevent_default();
                close_confirm();
            }
            "Escape" => {
                ev.prevent_default();
                close_cancel();
            }
            _ => {}
        }
    };

    let on_overlay_click = move |ev: ev::MouseEvent| {
        let Some(target) = ev.target() else {
            return;
        };
        let Ok(el) = target.dyn_into::<web_sys::Element>() else {
            return;
        };
        if el.class_list().contains("npp-overlay") {
            close_cancel();
        }
    };

    let open_popup = move |_| {
        popup_open.set(true);
    };

    let popup = move || {
        if !popup_open.get() {
            return None;
        }
        Some(view! {
            <div
                class="npp-overlay"
                role="dialog"
                aria-modal="true"
                aria-label="New page"
                on:click=on_overlay_click
            >
                <div class="npp-panel">
                    <div class="npp-preview" aria-hidden="true">
                        <For
                            each=move || letters.get()
                            key=|l| l.id
                            children=move |l: Letter| {
                                let style_attr = format!(
                                    "display: inline-block; animation: npp-shake {}ms cubic-bezier(0.22, 1, 0.36, 1) both; --npp-shake-amp: {}px;",
                                    SHAKE_DURATION_MS, SHAKE_AMPLITUDE_PX
                                );
                                view! { <span class="npp-letter" style=style_attr>{l.c.to_string()}</span> }
                            }
                        />
                    </div>
                    <input
                        node_ref=input_ref
                        class="npp-input"
                        type="text"
                        aria-label="Page title"
                        spellcheck="false"
                        autocomplete="off"
                        on:input=on_input
                        on:keydown=on_keydown
                    />
                    <div class="npp-hint">"Enter to create · Escape to cancel · click outside to cancel"</div>
                </div>
            </div>
        })
    };

    view! {
        <div class="npp-root" style=root_style>
            <h3 class="npp-workspace-title">"Workspace"</h3>
            <p class="npp-workspace-body">"Mock page content."</p>
            <button class="npp-btn" on:click=open_popup>"New page"</button>
            {move || match result.get() {
                Some(t) => view! {
                    <div class="npp-result">"Created page: " <strong>{t}</strong></div>
                }.into_any(),
                None => view! { <div></div> }.into_any(),
            }}
            {popup}
        </div>
    }
}