use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use std::time::Duration;
use wasm_bindgen::JsCast;

const TICK_MS: i32 = 100;
const MAX_LIVE: usize = 4;
const SPAWN_EVERY_MS: i32 = 2800;
const EXIT_MS: i32 = 380;
const CONSUMED_EXIT_MS: i32 = 220;
const CLOSE_MS: i32 = 240;
const MODAL_W: f64 = 560.0;

const SPAWN_JITTER_MS: &[i32] = &[0, 220, 80, 340, 140, 260, 60, 300];

const ICON_INFO: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/></svg>"#;
const ICON_SUCCESS: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M8 12l3 3 5-6"/></svg>"#;
const ICON_WARNING: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 3l10 18H2z"/><path d="M12 10v5"/><path d="M12 18h.01"/></svg>"#;
const ICON_ERROR: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M15 9l-6 6M9 9l6 6"/></svg>"#;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind { Info, Success, Warning, Error }

impl Kind {
    fn class(self) -> &'static str {
        match self { Kind::Info => "info", Kind::Success => "success", Kind::Warning => "warning", Kind::Error => "error" }
    }
    fn label(self) -> &'static str {
        match self { Kind::Info => "Info", Kind::Success => "Success", Kind::Warning => "Warning", Kind::Error => "Error" }
    }
    fn icon(self) -> &'static str {
        match self { Kind::Info => ICON_INFO, Kind::Success => ICON_SUCCESS, Kind::Warning => ICON_WARNING, Kind::Error => ICON_ERROR }
    }
    fn lifetime(self) -> i32 {
        match self { Kind::Info => 6000, Kind::Success => 5000, Kind::Warning => 8000, Kind::Error => 10000 }
    }
}

#[derive(Clone)]
enum DetailBlock {
    Para(&'static str),
    List(&'static [&'static str]),
    Log(&'static [&'static str]),
}

struct MsgDef {
    kind: Kind,
    title: &'static str,
    summary: &'static str,
}

const MESSAGES: &[MsgDef] = &[
    MsgDef { kind: Kind::Success, title: "Backup complete", summary: "1,284 pages · 312 MB · finished in 14 seconds" },
    MsgDef { kind: Kind::Error, title: "Sync failed for Trip itinerary", summary: "Retried 3 times, will retry on reconnect" },
    MsgDef { kind: Kind::Warning, title: "Storage almost full, 91% used", summary: "38.2 GB of 42 GB across 4 workspaces" },
    MsgDef { kind: Kind::Info, title: "Version 3.12 is available", summary: "22 changes, including faster block reorder" },
    MsgDef { kind: Kind::Warning, title: "Offline, changes are queued", summary: "7 edits waiting to sync" },
    MsgDef { kind: Kind::Error, title: "Could not upload 2 images", summary: "Retry, or replace the files" },
    MsgDef { kind: Kind::Success, title: "Export ready: Budget 2026.pdf", summary: "24 pages, saved to your desktop" },
    MsgDef { kind: Kind::Success, title: "Merged 14 duplicate blocks", summary: "Across 3 pages" },
    MsgDef { kind: Kind::Info, title: "Weekly review is due tomorrow", summary: "Your reminder, no action needed" },
    MsgDef { kind: Kind::Info, title: "3 collaborators joined", summary: "Mira, Jonas, and Priya" },
];

fn details(idx: usize) -> Vec<DetailBlock> {
    match idx {
        0 => vec![
            DetailBlock::Para("Your notes were backed up just now. Nothing else needs doing."),
            DetailBlock::List(&["1,284 pages", "312 MB total", "Last backup: 4 minutes ago"]),
        ],
        1 => vec![
            DetailBlock::Para("Sync failed after 3 retries. The page was saved locally and will retry when you reconnect."),
            DetailBlock::Log(&[
                "08:41:12 INFO  opening session ...",
                "08:41:12 INFO  session id 7e2b4c01",
                "08:41:12 INFO  local db ready",
                "08:41:13 INFO  requesting page Trip itinerary",
                "08:41:13 INFO  found local blocks=14",
                "08:41:13 INFO  connecting to sync server",
                "08:41:14 WARN  connection attempt 1 timed out",
                "08:41:15 WARN  connection attempt 2 timed out",
                "08:41:17 WARN  connection attempt 3 timed out",
                "08:41:18 ERROR could not reach sync server",
                "08:41:18 ERROR sync aborted for Trip itinerary",
                "08:41:18 INFO  falling back to offline mode",
                "08:41:18 INFO  queued 3 edits for later sync",
                "08:41:18 INFO  next retry scheduled in 30 seconds",
                "08:41:18 INFO  session preserved for reconnect",
                "08:41:18 INFO  keeping local revision r=17",
            ]),
        ],
        2 => vec![
            DetailBlock::Para("Storage is at 91%. You can free space by archiving old pages or removing images."),
            DetailBlock::List(&[
                "Weekly planning — 12.4 GB",
                "Trip itinerary — 8.1 GB",
                "Recipes to try — 6.8 GB",
                "Reading list — 5.3 GB",
                "Meeting notes — 2.1 GB",
                "Budget 2026 — 1.8 GB",
                "Old drafts — 1.2 GB",
                "Images in Recipes — 0.5 GB",
            ]),
            DetailBlock::Para("Archive a workspace from settings > storage to reclaim most of its space."),
        ],
        3 => vec![
            DetailBlock::Para("An update is ready. Restart the app to apply it."),
            DetailBlock::List(&[
                "Faster block reorder when the page has more than 50 blocks",
                "Smaller bundle size (minus 12%)",
                "Fix: caret no longer jumps after a merge",
                "Fix: disabled pages stay out of the search",
                "New: keyboard shortcut for 'Split block' (Enter)",
                "New: per-page search history",
                "Polish: smoother drag into the vacuum strip",
                "Polish: softer idle page sway",
                "Fix: crash when uploading a large image",
                "Fix: sync conflict on rapid editing",
                "New: 'Change pick' button on the two-version page",
                "Fix: autosave no longer stops after a modal",
            ]),
        ],
        4 => vec![
            DetailBlock::Para("You are offline. Changes are saved locally and will sync when you reconnect."),
            DetailBlock::List(&["Weekly planning — 3 edits", "Trip itinerary — 2 edits", "Budget 2026 — 2 edits"]),
        ],
        5 => vec![
            DetailBlock::Para("Two image uploads failed. You can retry or replace the files."),
            DetailBlock::List(&["sunset.jpg — file too large (12 MB, limit 8 MB)", "map.png — unsupported format"]),
        ],
        6 => vec![DetailBlock::Para("Exported to Budget 2026.pdf on your desktop.")],
        7 => vec![DetailBlock::Para("Found 14 duplicate blocks across 3 pages and merged them.")],
        8 => vec![DetailBlock::Para("Your weekly review is due tomorrow. Open the Weekly review page to start.")],
        9 => vec![DetailBlock::Para("Mira, Jonas, and Priya joined this workspace.")],
        _ => vec![DetailBlock::Para("Nothing more to say.")],
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ToastState { Live, Leaving(ExitKind) }

#[derive(Clone, Copy, PartialEq, Eq)]
enum ExitKind { Auto, Dismiss, Consumed }

#[derive(Clone)]
struct Toast {
    id: u64,
    msg_idx: usize,
    life_ms: i32,
    remaining: RwSignal<i32>,
    state: RwSignal<ToastState>,
    leave_left: RwSignal<i32>,
}

#[derive(Clone, Copy)]
struct ModalState {
    #[allow(dead_code)]
    gen_id: u64,
    msg_idx: usize,
    fx: f64,
    fy: f64,
    fs: f64,
}

#[component]
pub fn Popups() -> impl IntoView {
    let toasts = RwSignal::new(Vec::<Toast>::new());
    let next_id = RwSignal::new(1u64);
    let msg_cursor = RwSignal::new(0usize);
    let spawn_counter = RwSignal::new(0u64);
    let hover_id = RwSignal::new(None::<u64>);
    let auto_spawn = RwSignal::new(true);
    let spawn_left = RwSignal::new(SPAWN_EVERY_MS);
    let modal = RwSignal::new(None::<ModalState>);
    let modal_closing = RwSignal::new(false);
    let modal_left = RwSignal::new(0i32);
    let next_modal_gen = RwSignal::new(1u64);

    let begin_leave = Callback::new(move |(t, kind): (Toast, ExitKind)| {
        if matches!(t.state.get_untracked(), ToastState::Live) {
            let dur = match kind { ExitKind::Consumed => CONSUMED_EXIT_MS, _ => EXIT_MS };
            t.state.set(ToastState::Leaving(kind));
            t.leave_left.set(dur);
        }
    });

    let spawn = Callback::new(move |_: ()| {
        if modal.get_untracked().is_some() { return; }
        let live_count = toasts.with_untracked(|v| v.iter().filter(|t| matches!(t.state.get_untracked(), ToastState::Live)).count());
        if live_count >= MAX_LIVE {
            let oldest_live = toasts.with_untracked(|v| v.iter().find(|t| matches!(t.state.get_untracked(), ToastState::Live)).cloned());
            if let Some(ot) = oldest_live { begin_leave.run((ot, ExitKind::Auto)); }
        }
        let idx = msg_cursor.get_untracked();
        msg_cursor.set((idx + 1) % MESSAGES.len());
        let id = next_id.get_untracked();
        next_id.set(id + 1);
        let life = MESSAGES[idx].kind.lifetime();
        toasts.update(|v| v.push(Toast {
            id, msg_idx: idx, life_ms: life,
            remaining: RwSignal::new(life),
            state: RwSignal::new(ToastState::Live),
            leave_left: RwSignal::new(0),
        }));
        spawn_counter.update(|c| *c += 1);
    });

    Effect::new(move |_| {
        spawn.run(());
        spawn.run(());
    });

    Effect::new(move |_| {
        let h = set_interval_with_handle(move || {
            if let Some(hid) = hover_id.get_untracked() {
                let alive = toasts.with_untracked(|v| v.iter().any(|t| t.id == hid));
                if !alive { hover_id.set(None); }
            }
            let paused = modal.get_untracked().is_some() || hover_id.get_untracked().is_some();
            let snapshot = toasts.get_untracked();
            if !paused {
                for t in &snapshot {
                    if matches!(t.state.get_untracked(), ToastState::Live) {
                        let r = t.remaining.get_untracked() - TICK_MS;
                        if r <= 0 { begin_leave.run((t.clone(), ExitKind::Auto)); }
                        else { t.remaining.set(r); }
                    }
                }
            }
            let mut to_remove: Vec<u64> = Vec::new();
            for t in &snapshot {
                if let ToastState::Leaving(_) = t.state.get_untracked() {
                    let l = t.leave_left.get_untracked() - TICK_MS;
                    if l <= 0 { to_remove.push(t.id); }
                    else { t.leave_left.set(l); }
                }
            }
            if auto_spawn.get_untracked() && !paused {
                let s = spawn_left.get_untracked() - TICK_MS;
                if s <= 0 {
                    spawn.run(());
                    let j = SPAWN_JITTER_MS[(spawn_counter.get_untracked() as usize) % SPAWN_JITTER_MS.len()];
                    spawn_left.set(SPAWN_EVERY_MS + j);
                } else { spawn_left.set(s); }
            }
            if modal_closing.get_untracked() {
                let m = modal_left.get_untracked() - TICK_MS;
                if m <= 0 { modal.set(None); modal_closing.set(false); }
                else { modal_left.set(m); }
            }
            if !to_remove.is_empty() {
                toasts.update(|v| v.retain(|t| !to_remove.contains(&t.id)));
            }
        }, Duration::from_millis(TICK_MS as u64));
        match h {
            Ok(handle) => on_cleanup(move || handle.clear()),
            Err(e) => leptos::logging::error!("popups: tick: {e:?}"),
        }
    });

    let begin_close = Callback::new(move |_: ()| {
        if modal.get_untracked().is_none() || modal_closing.get_untracked() { return; }
        modal_closing.set(true);
        modal_left.set(CLOSE_MS);
    });

    Effect::new(move |_| {
        let h = window_event_listener(ev::keydown, move |e: ev::KeyboardEvent| {
            if e.key() == "Escape" { begin_close.run(()); }
        });
        on_cleanup(move || h.remove());
    });

    let stack_class = move || {
        if modal.get().is_some() || hover_id.get().is_some() { "pp-stack held" } else { "pp-stack" }
    };

    view! {
        <div class="pp-root">
            <div class="pp-toolbar">
                <button class="btn" on:click=move |_| spawn.run(())>"Spawn popup"</button>
                <button class="btn" aria-pressed=move || if auto_spawn.get() { "true" } else { "false" }
                    on:click=move |_| auto_spawn.update(|v| *v = !*v)>
                    {move || if auto_spawn.get() { "Auto-spawn: on" } else { "Auto-spawn: off" }}
                </button>
                <span class="label">"Popups appear in the bottom-right corner of the window. Click one to open it."</span>
            </div>
            <div class="mock-page">
                <div class="mock-title">"Workspace activity"</div>
                <div class="mock-block">"A notebook with three pages open. Small events land in the corner as popups; click one to inspect it."</div>
                <div class="mock-block">"While a popup is open as a modal, the rest of the stack is held. Nothing disappears behind the panel."</div>
                <div class="mock-block">"Hover any toast and the stack pauses. Click any toast to open it."</div>
            </div>
            <div class=stack_class aria-live="polite">
                <For
                    each=move || toasts.get()
                    key=|t| t.id
                    children=move |t: Toast| {
                        render_toast(t, hover_id, modal, next_modal_gen, begin_leave)
                    }
                />
            </div>
            {move || modal.get().map(|m| render_modal(m, modal_closing, begin_close))}
        </div>
    }
}

fn render_toast(
    t: Toast,
    hover_id: RwSignal<Option<u64>>,
    modal: RwSignal<Option<ModalState>>,
    next_modal_gen: RwSignal<u64>,
    begin_leave: Callback<(Toast, ExitKind)>,
) -> impl IntoView {
    let id = t.id;
    let msg_idx = t.msg_idx;
    let msg = &MESSAGES[msg_idx];
    let kind = msg.kind;
    let title = msg.title;
    let summary = msg.summary;
    let icon = kind.icon();
    let kind_class = kind.class();
    let life = t.life_ms;
    let remaining = t.remaining;
    let state = t.state;
    let toast_ref = NodeRef::<html::Div>::new();
    let t_for_main = t.clone();
    let t_for_dismiss = t.clone();

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
        let pct = (remaining.get() as f64 / life as f64).clamp(0.0, 1.0);
        format!("--pct: {:.3}", pct)
    };
    let on_main = move |_| {
        if modal.get_untracked().is_some() { return; }
        if let Some(el) = toast_ref.get_untracked() {
            let rect = el.get_bounding_client_rect();
            let win = web_sys::window();
            let vw = win.as_ref().and_then(|w| w.inner_width().ok()).and_then(|v| v.as_f64()).unwrap_or(1024.0);
            let vh = win.as_ref().and_then(|w| w.inner_height().ok()).and_then(|v| v.as_f64()).unwrap_or(768.0);
            let cx = rect.left() + rect.width() / 2.0;
            let cy = rect.top() + rect.height() / 2.0;
            let fx = cx - vw / 2.0;
            let fy = cy - vh / 2.0;
            let mw = MODAL_W.min(0.92 * vw);
            let fs = (rect.width() / mw).clamp(0.3, 1.0);
            let gen_id = next_modal_gen.get_untracked();
            next_modal_gen.set(gen_id + 1);
            begin_leave.run((t_for_main.clone(), ExitKind::Consumed));
            modal.set(Some(ModalState { gen_id, msg_idx, fx, fy, fs }));
        }
    };
    let on_x = move |ev: ev::MouseEvent| {
        ev.stop_propagation();
        begin_leave.run((t_for_dismiss.clone(), ExitKind::Dismiss));
    };
    view! {
        <div class=row_class role="status"
            on:pointerenter=move |_| hover_id.set(Some(id))
            on:pointerleave=move |_| {
                if hover_id.get_untracked() == Some(id) { hover_id.set(None); }
            }>
            <div class="pp-row-clip">
                <div class=toast_class node_ref=toast_ref>
                    <button class="pp-toast-main" on:click=on_main>
                        <span class=format!("pp-icon pp-icon-{}", kind_class) inner_html=icon></span>
                        <span class="pp-toast-text">
                            <span class="pp-toast-title">{title}</span>
                            <span class="pp-toast-summary">{summary}</span>
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
    let msg = &MESSAGES[m.msg_idx];
    let kind = msg.kind;
    let title = msg.title;
    let icon = kind.icon();
    let kind_class = kind.class();
    let kind_label = kind.label();
    let panel_style = format!(
        "--pp-fx: {:.1}px; --pp-fy: {:.1}px; --pp-fs: {:.3};",
        m.fx, m.fy, m.fs
    );
    let detail = details(m.msg_idx);
    let detail_view = detail.into_iter().map(render_detail).collect_view();

    let overlay_class = move || if modal_closing.get() { "pp-overlay closing" } else { "pp-overlay" };
    let panel_class = move || if modal_closing.get() { "pp-panel closing" } else { "pp-panel" };
    let close_btn_ref = NodeRef::<html::Button>::new();
    Effect::new(move |_| {
        set_timeout(move || {
            if let Some(b) = close_btn_ref.get() { let _ = b.focus(); }
        }, Duration::from_millis(0));
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
                    <span class=format!("pp-icon pp-icon-{}", kind_class) inner_html=icon></span>
                    <span class="pp-panel-title" id="pp-modal-title">{title}</span>
                    <span class=format!("label pp-kind-label pp-kind-{}", kind_class)>{kind_label}</span>
                    <button class="pp-panel-x" node_ref=close_btn_ref aria-label="Close"
                        on:click=move |_| begin_close.run(())>"\u{00d7}"</button>
                </div>
                <div class="pp-panel-meta"><span class="label">"Just now"</span></div>
                <div class="pp-panel-body" tabindex="0">{detail_view}</div>
                <div class="pp-panel-foot">
                    <span class="label">"Esc to close"</span>
                    <button class="btn" on:click=move |_| begin_close.run(())>"Close"</button>
                </div>
            </div>
        </div>
    }.into_any()
}

fn render_detail(block: DetailBlock) -> AnyView {
    match block {
        DetailBlock::Para(s) => view! { <p>{s}</p> }.into_any(),
        DetailBlock::List(items) => view! {
            <ul class="pp-list">
                {items.iter().map(|s| view! { <li>{*s}</li> }).collect_view()}
            </ul>
        }.into_any(),
        DetailBlock::Log(lines) => {
            let text = lines.join("\n");
            view! { <pre class="pp-log">{text}</pre> }.into_any()
        }
    }
}