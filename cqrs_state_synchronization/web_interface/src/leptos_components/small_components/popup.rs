use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use wasm_bindgen::JsCast;

const TICK_MS: i32 = 100;
const EXIT_MS: i32 = 380;
const CONSUMED_EXIT_MS: i32 = 220;
const CLOSE_MS: i32 = 240;
const MODAL_W: f64 = 560.0;

const ICON_INFO: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/></svg>"#;
const ICON_SUCCESS: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M8 12l3 3 5-6"/></svg>"#;
const ICON_WARNING: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 3l10 18H2z"/><path d="M12 10v5"/><path d="M12 18h.01"/></svg>"#;
const ICON_ERROR: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M15 9l-6 6M9 9l6 6"/></svg>"#;

const CSS: &str = r#"
.pp-stack { position: fixed; left: 20px; bottom: 20px; width: min(360px, calc(100vw - 40px)); display: flex; flex-direction: column; z-index: 90; pointer-events: none; }
.pp-row { display: grid; grid-template-rows: 1fr; pointer-events: auto; animation: pp-row-enter 260ms cubic-bezier(0.22, 1, 0.36, 1) backwards; transition: grid-template-rows 240ms cubic-bezier(0.22, 1, 0.36, 1) 140ms; }
.pp-row.leaving { grid-template-rows: 0fr; pointer-events: none; animation: none; }
.pp-row.consumed { grid-template-rows: 0fr; pointer-events: none; animation: none; transition: grid-template-rows 220ms cubic-bezier(0.22, 1, 0.36, 1) 0ms; }
@keyframes pp-row-enter { from { grid-template-rows: 0fr; } to { grid-template-rows: 1fr; } }
.pp-row-clip { min-height: 0; padding: 0 16px; margin: 0 -16px; }
.pp-row.leaving .pp-row-clip, .pp-row.consumed .pp-row-clip { overflow: hidden; }
.pp-toast { position: relative; margin-top: 10px; background: #151c29; border: 1px solid rgba(228, 233, 242, 0.09); border-left: 3px solid #8ab4ff; border-radius: 6px; box-shadow: 0 8px 24px -12px rgba(0,0,0,0.6); overflow: hidden; animation: pp-toast-enter 380ms cubic-bezier(0.34, 1.56, 0.64, 1) 40ms backwards, pp-fade-enter 180ms ease-out 40ms backwards; color: #e4e9f2; font-family: system-ui, -apple-system, sans-serif; }
.pp-toast-info { border-left-color: rgba(228, 233, 242, 0.72); }
.pp-toast-success { border-left-color: #8ab4ff; }
.pp-toast-warning { border-left-color: #f2b45a; }
.pp-toast-error { border-left-color: #ff6b6b; }
@keyframes pp-toast-enter { from { transform: translateX(-28px) scale(0.97); } to { transform: translateX(0) scale(1); } }
@keyframes pp-fade-enter { from { opacity: 0; } to { opacity: 1; } }
.pp-row.leaving .pp-toast { animation: pp-leave 240ms cubic-bezier(0.22, 1, 0.36, 1) forwards; }
@keyframes pp-leave { from { opacity: 1; transform: translateX(0); } to { opacity: 0; transform: translateX(-32px); } }
.pp-row.consumed .pp-toast { animation: pp-consume 90ms linear forwards; }
@keyframes pp-consume { to { opacity: 0; } }
.pp-toast-main { display: flex; align-items: flex-start; gap: 10px; padding: 10px 12px 12px; background: transparent; border: none; text-align: left; font: inherit; width: 100%; color: inherit; cursor: pointer; }
.pp-toast-main:hover .pp-toast-title { color: #8ab4ff; }
.pp-icon { flex-shrink: 0; display: inline-flex; width: 16px; height: 16px; align-items: center; justify-content: center; margin-top: 1px; }
.pp-icon svg { width: 16px; height: 16px; }
.pp-icon-info { color: rgba(228, 233, 242, 0.72); }
.pp-icon-success { color: #8ab4ff; }
.pp-icon-warning { color: #f2b45a; }
.pp-icon-error { color: #ff6b6b; }
.pp-toast-text { display: flex; flex-direction: column; gap: 2px; min-width: 0; flex: 1; }
.pp-toast-title { font-size: 13px; font-weight: 600; color: #e4e9f2; transition: color 120ms ease; word-break: break-word; }
.pp-toast-x { position: absolute; top: 4px; right: 4px; background: transparent; border: none; color: rgba(228, 233, 242, 0.42); font-size: 16px; line-height: 1; padding: 2px 6px; cursor: pointer; border-radius: 4px; opacity: 0; transition: opacity 120ms ease, color 120ms ease, background 120ms ease; }
.pp-row:hover .pp-toast-x, .pp-toast-x:focus-visible { opacity: 1; }
.pp-toast-x:hover { color: #e4e9f2; background: #2b3648; }
.pp-timer { position: absolute; bottom: 0; left: 0; right: 0; height: 2px; background: #8ab4ff; transform-origin: left; transform: scaleX(var(--pct, 1)); transition: transform 100ms linear, background 200ms ease; }
.pp-timer-info { background: rgba(228, 233, 242, 0.72); }
.pp-timer-success { background: #8ab4ff; }
.pp-timer-warning { background: #f2b45a; }
.pp-timer-error { background: #ff6b6b; }
.pp-stack.held .pp-timer { background: rgba(228, 233, 242, 0.42); }
.pp-overlay { position: fixed; inset: 0; z-index: 100; background: rgba(10, 14, 24, 0.6); display: flex; align-items: center; justify-content: center; padding: 24px; box-sizing: border-box; animation: pp-fade-in 200ms ease both; font-family: system-ui, -apple-system, sans-serif; }
@keyframes pp-fade-in { from { opacity: 0; } to { opacity: 1; } }
.pp-overlay.closing { animation: pp-fade-out 200ms ease forwards; pointer-events: none; }
@keyframes pp-fade-out { to { opacity: 0; } }
.pp-panel { width: min(560px, 92vw); max-height: min(80vh, 680px); display: flex; flex-direction: column; background: #1c2433; border: 1px solid rgba(228, 233, 242, 0.09); border-radius: 14px; box-shadow: 0 24px 48px -18px rgba(0,0,0,0.75); animation: pp-flight-in 380ms cubic-bezier(0.22, 1, 0.36, 1) both, pp-fade-in 140ms ease-out both; color: #e4e9f2; }
.pp-panel.closing { animation: pp-panel-close 200ms cubic-bezier(0.22, 1, 0.36, 1) forwards; pointer-events: none; }
@keyframes pp-flight-in { from { transform: translate(var(--pp-fx, 0), var(--pp-fy, 0)) scale(var(--pp-fs, 1)); } to { transform: none; } }
@keyframes pp-panel-close { to { opacity: 0; transform: translateY(10px) scale(0.97); } }
.pp-panel-head { display: flex; align-items: center; gap: 10px; padding: 14px 18px; border-bottom: 1px solid rgba(228, 233, 242, 0.09); flex-shrink: 0; }
.pp-panel-title { font-size: 15px; font-weight: 600; color: #e4e9f2; flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.pp-kind-label { flex-shrink: 0; font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.12em; }
.pp-kind-info { color: rgba(228, 233, 242, 0.72); }
.pp-kind-success { color: #8ab4ff; }
.pp-kind-warning { color: #f2b45a; }
.pp-kind-error { color: #ff6b6b; }
.pp-panel-x { background: transparent; border: none; color: rgba(228, 233, 242, 0.42); font-size: 22px; line-height: 1; padding: 2px 8px; cursor: pointer; border-radius: 6px; flex-shrink: 0; transition: color 120ms ease, background 120ms ease; }
.pp-panel-x:hover { color: #e4e9f2; background: #2b3648; }
.pp-panel-meta { padding: 8px 18px; border-bottom: 1px solid rgba(228, 233, 242, 0.09); flex-shrink: 0; font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.12em; color: rgba(228, 233, 242, 0.42); }
.pp-panel-body { flex: 1; min-height: 0; overflow-y: auto; overscroll-behavior: contain; padding: 16px 18px; color: rgba(228, 233, 242, 0.72); font-size: 14px; line-height: 1.6; white-space: pre-wrap; word-break: break-word; scrollbar-width: thin; scrollbar-color: rgba(228,233,242,0.2) transparent; }
.pp-panel-body:focus-visible { outline: none; }
.pp-panel-foot { padding: 12px 18px; border-top: 1px solid rgba(228, 233, 242, 0.09); display: flex; justify-content: space-between; align-items: center; flex-shrink: 0; font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.12em; color: rgba(228, 233, 242, 0.42); }
.pp-panel-foot .btn { font-size: 13px; padding: 6px 12px; border: 1px solid rgba(228, 233, 242, 0.09); border-radius: 6px; background: transparent; color: rgba(228, 233, 242, 0.72); cursor: pointer; font-family: inherit; text-transform: none; letter-spacing: normal; font-weight: normal; }
.pp-panel-foot .btn:hover { border-color: rgba(228, 233, 242, 0.2); background: #2b3648; color: #e4e9f2; }
@media (prefers-reduced-motion: reduce) { .pp-row { animation: pp-fade-enter 150ms ease backwards; } .pp-row.leaving, .pp-row.consumed { display: none; animation: none; } .pp-toast { animation: pp-fade-enter 150ms ease backwards; } .pp-timer { transition: none; } .pp-overlay, .pp-panel { animation: pp-fade-in 150ms ease both; } .pp-overlay.closing, .pp-panel.closing { animation: pp-fade-out 150ms ease forwards; } }
"#;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PopupKind {
    Info,
    Success,
    Warning,
    Error,
}

impl PopupKind {
    fn class(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Success => "success",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Info => "Info",
            Self::Success => "Success",
            Self::Warning => "Warning",
            Self::Error => "Error",
        }
    }
    fn icon(self) -> &'static str {
        match self {
            Self::Info => ICON_INFO,
            Self::Success => ICON_SUCCESS,
            Self::Warning => ICON_WARNING,
            Self::Error => ICON_ERROR,
        }
    }
    fn lifetime(self) -> i32 {
        match self {
            Self::Info => 6000,
            Self::Success => 5000,
            Self::Warning => 8000,
            Self::Error => 10000,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ToastState {
    Live,
    Leaving(ExitKind),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ExitKind {
    Auto,
    Dismiss,
    Consumed,
}

#[derive(Clone)]
struct Popup {
    id: u64,
    kind: PopupKind,
    message: String,
    life_ms: i32,
    remaining: ArcRwSignal<i32>,
    state: ArcRwSignal<ToastState>,
    leave_left: ArcRwSignal<i32>,
}

static POPUPS: LazyLock<ArcRwSignal<Vec<Popup>>> = LazyLock::new(|| ArcRwSignal::new(Vec::new()));
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub fn show_popup(message: impl Into<String>) {
    show_popup_as(PopupKind::Info, message);
}

pub fn show_popup_as(kind: PopupKind, message: impl Into<String>) {
    let message: String = message.into();
    let mut coalesced = false;
    POPUPS.update(|v| {
        for p in v.iter_mut() {
            if p.kind == kind
                && p.message == message
                && matches!(p.state.get_untracked(), ToastState::Live)
            {
                p.remaining.set(p.life_ms);
                coalesced = true;
                return;
            }
        }
    });
    if coalesced {
        return;
    }
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let life = kind.lifetime();
    let popup = Popup {
        id,
        kind,
        message,
        life_ms: life,
        remaining: ArcRwSignal::new(life),
        state: ArcRwSignal::new(ToastState::Live),
        leave_left: ArcRwSignal::new(0),
    };
    POPUPS.update(|v| v.push(popup));
}

#[derive(Clone)]
struct ModalState {
    kind: PopupKind,
    message: String,
    fx: f64,
    fy: f64,
    fs: f64,
}

#[component]
pub fn CoolPopup() -> impl IntoView {
    let hover_id = RwSignal::new(None::<u64>);
    let modal = RwSignal::new(None::<ModalState>);
    let modal_closing = RwSignal::new(false);
    let modal_left = RwSignal::new(0i32);

    let begin_leave = Callback::new(move |(p, kind): (Popup, ExitKind)| {
        if matches!(p.state.get_untracked(), ToastState::Live) {
            let dur = match kind {
                ExitKind::Consumed => CONSUMED_EXIT_MS,
                _ => EXIT_MS,
            };
            p.state.set(ToastState::Leaving(kind));
            p.leave_left.set(dur);
        }
    });

    Effect::new(move |_| {
        let h = set_interval_with_handle(
            move || {
                if let Some(hid) = hover_id.get_untracked() {
                    let alive = POPUPS.with_untracked(|v| v.iter().any(|p| p.id == hid));
                    if !alive {
                        hover_id.set(None);
                    }
                }
                let paused = modal.get_untracked().is_some() || hover_id.get_untracked().is_some();
                let snapshot = POPUPS.get_untracked();
                if !paused {
                    for p in &snapshot {
                        if matches!(p.state.get_untracked(), ToastState::Live) {
                            let r = p.remaining.get_untracked() - TICK_MS;
                            if r <= 0 {
                                begin_leave.run((p.clone(), ExitKind::Auto));
                            } else {
                                p.remaining.set(r);
                            }
                        }
                    }
                }
                let mut to_remove: Vec<u64> = Vec::new();
                for p in &snapshot {
                    if let ToastState::Leaving(_) = p.state.get_untracked() {
                        let l = p.leave_left.get_untracked() - TICK_MS;
                        if l <= 0 {
                            to_remove.push(p.id);
                        } else {
                            p.leave_left.set(l);
                        }
                    }
                }
                if modal_closing.get_untracked() {
                    let m = modal_left.get_untracked() - TICK_MS;
                    if m <= 0 {
                        modal.set(None);
                        modal_closing.set(false);
                    } else {
                        modal_left.set(m);
                    }
                }
                if !to_remove.is_empty() {
                    POPUPS.update(|v| v.retain(|p| !to_remove.contains(&p.id)));
                }
            },
            Duration::from_millis(TICK_MS as u64),
        );
        match h {
            Ok(handle) => on_cleanup(move || handle.clear()),
            Err(e) => leptos::logging::error!("cool_popup: tick: {e:?}"),
        }
    });

    let begin_close = Callback::new(move |_: ()| {
        if modal.get_untracked().is_none() || modal_closing.get_untracked() {
            return;
        }
        modal_closing.set(true);
        modal_left.set(CLOSE_MS);
    });

    Effect::new(move |_| {
        let h = window_event_listener(ev::keydown, move |e: ev::KeyboardEvent| {
            if e.key() == "Escape" {
                begin_close.run(());
            }
        });
        on_cleanup(move || h.remove());
    });

    let stack_class = move || {
        if modal.get().is_some() || hover_id.get().is_some() {
            "pp-stack held"
        } else {
            "pp-stack"
        }
    };

    view! {
        <style>{CSS}</style>
        <div class=stack_class aria-live="polite">
            <For
                each=move || POPUPS.get()
                key=|p| p.id
                children=move |p: Popup| render_toast(p, hover_id, modal, begin_leave)
            />
        </div>
        {move || modal.get().map(|m| render_modal(m, modal_closing, begin_close))}
    }
}

fn render_toast(
    p: Popup,
    hover_id: RwSignal<Option<u64>>,
    modal: RwSignal<Option<ModalState>>,
    begin_leave: Callback<(Popup, ExitKind)>,
) -> impl IntoView {
    let id = p.id;
    let kind = p.kind;
    let kind_class = kind.class();
    let kind_icon = kind.icon();
    let life_ms = p.life_ms;
    let remaining = p.remaining.clone();
    let state = p.state.clone();
    let message = p.message.clone();
    let message_for_main = message.clone();
    let toast_ref = NodeRef::<html::Div>::new();
    let p_main = p.clone();
    let p_dismiss = p;

    let row_class = move || {
        let mut c = String::from("pp-row");
        match state.get() {
            ToastState::Live => {}
            ToastState::Leaving(ExitKind::Consumed) => c.push_str(" leaving consumed"),
            ToastState::Leaving(_) => c.push_str(" leaving"),
        }
        c
    };
    let toast_class = format!("pp-toast pp-toast-{}", kind_class);
    let timer_style = move || {
        let pct = (remaining.get() as f64 / life_ms as f64).clamp(0.0, 1.0);
        format!("--pct: {:.3}", pct)
    };
    let on_main = move |_| {
        if modal.get_untracked().is_some() {
            return;
        }
        if let Some(el) = toast_ref.get_untracked() {
            let rect = el.get_bounding_client_rect();
            let win = web_sys::window();
            let vw = win
                .as_ref()
                .and_then(|w| w.inner_width().ok())
                .and_then(|v| v.as_f64())
                .unwrap_or(1024.0);
            let vh = win
                .as_ref()
                .and_then(|w| w.inner_height().ok())
                .and_then(|v| v.as_f64())
                .unwrap_or(768.0);
            let cx = rect.left() + rect.width() / 2.0;
            let cy = rect.top() + rect.height() / 2.0;
            let fx = cx - vw / 2.0;
            let fy = cy - vh / 2.0;
            let mw = MODAL_W.min(0.92 * vw);
            let fs = (rect.width() / mw).clamp(0.3, 1.0);
            begin_leave.run((p_main.clone(), ExitKind::Consumed));
            modal.set(Some(ModalState {
                kind,
                message: message_for_main.clone(),
                fx,
                fy,
                fs,
            }));
        }
    };
    let on_x = move |ev: ev::MouseEvent| {
        ev.stop_propagation();
        begin_leave.run((p_dismiss.clone(), ExitKind::Dismiss));
    };
    view! {
        <div class=row_class role="status"
            on:pointerenter=move |_| hover_id.set(Some(id))
            on:pointerleave=move |_| {
                if hover_id.get_untracked() == Some(id) {
                    hover_id.set(None);
                }
            }>
            <div class="pp-row-clip">
                <div class=toast_class node_ref=toast_ref>
                    <button class="pp-toast-main" on:click=on_main>
                        <span class=format!("pp-icon pp-icon-{}", kind_class) inner_html=kind_icon></span>
                        <span class="pp-toast-text">
                            <span class="pp-toast-title">{message}</span>
                        </span>
                    </button>
                    <button class="pp-toast-x" aria-label="Dismiss" on:click=on_x>"\u{00d7}"</button>
                    <div class=format!("pp-timer pp-timer-{}", kind_class) style=timer_style></div>
                </div>
            </div>
        </div>
    }
}

fn render_modal(
    m: ModalState,
    modal_closing: RwSignal<bool>,
    begin_close: Callback<()>,
) -> AnyView {
    let kind = m.kind;
    let kind_class = kind.class();
    let kind_icon = kind.icon();
    let kind_label = kind.label();
    let message = m.message;
    let panel_style = format!(
        "--pp-fx: {:.1}px; --pp-fy: {:.1}px; --pp-fs: {:.3};",
        m.fx, m.fy, m.fs
    );
    let overlay_class = move || {
        if modal_closing.get() {
            "pp-overlay closing"
        } else {
            "pp-overlay"
        }
    };
    let panel_class = move || {
        if modal_closing.get() {
            "pp-panel closing"
        } else {
            "pp-panel"
        }
    };
    let close_btn_ref = NodeRef::<html::Button>::new();
    Effect::new(move |_| {
        set_timeout(
            move || {
                if let Some(b) = close_btn_ref.get() {
                    let _ = b.focus();
                }
            },
            Duration::from_millis(0),
        );
    });
    let on_overlay_click = move |ev: ev::MouseEvent| {
        if let Some(target) = ev.target() {
            if let Ok(el) = target.dyn_into::<web_sys::Element>() {
                if el.class_list().contains("pp-overlay") {
                    begin_close.run(());
                }
            }
        }
    };
    view! {
        <div class=overlay_class on:click=on_overlay_click>
            <div class=panel_class style=panel_style role="dialog" aria-modal="true" aria-labelledby="pp-modal-title">
                <div class="pp-panel-head">
                    <span class=format!("pp-icon pp-icon-{}", kind_class) inner_html=kind_icon></span>
                    <span class="pp-panel-title" id="pp-modal-title">{kind_label}</span>
                    <span class=format!("pp-kind-label pp-kind-{}", kind_class)>{kind_label}</span>
                    <button class="pp-panel-x" node_ref=close_btn_ref aria-label="Close"
                        on:click=move |_| begin_close.run(())>"\u{00d7}"</button>
                </div>
                <div class="pp-panel-meta">"Just now"</div>
                <div class="pp-panel-body" tabindex="0">{message}</div>
                <div class="pp-panel-foot">
                    <span>"Esc to close"</span>
                    <button class="btn" on:click=move |_| begin_close.run(())>"Close"</button>
                </div>
            </div>
        </div>
    }.into_any()
}