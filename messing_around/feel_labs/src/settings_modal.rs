use leptos::ev;
use leptos::prelude::*;
use std::time::Duration;
use wasm_bindgen::JsCast;

const SAVED_BEAT_MS: u32 = 650;
const CLOSE_MS: u32 = 160;
const NAV_BTN_H: u32 = 34;
const NAV_GAP: u32 = 2;

const SHORTCUTS: &[(&str, &str)] = &[
    ("New page", "Ctrl+N"),
    ("Search", "Ctrl+K"),
    ("Toggle sidebar", "Ctrl+B"),
    ("Settings", "Ctrl+,"),
    ("Save", "Ctrl+S"),
    ("Undo", "Ctrl+Z"),
    ("Redo", "Ctrl+Shift+Z"),
    ("Split block", "Enter"),
    ("Merge with above", "Backspace"),
    ("Close modal", "Escape"),
];

const THEME_OPTIONS: &[(&str, i32)] = &[("Midnight", 0), ("Slate", 1), ("Contrast", 2)];
const ACCENT_OPTIONS: &[(&str, &str)] = &[
    ("Periwinkle", "var(--accent)"),
    ("Slate", "var(--fg-dim)"),
    ("Amber", "var(--warn)"),
    ("Coral", "var(--danger)"),
    ("Mist", "color-mix(in srgb, var(--accent) 45%, var(--fg))"),
];
const DENSITY_OPTIONS: &[(&str, &str, i32)] = &[
    ("Comfortable", "Roomier spacing throughout.", 0),
    ("Compact", "Tighter rows so more fits on screen.", 1),
];
const RADIUS_OPTIONS: &[(&str, i32)] = &[("Sharp", 0), ("Soft", 1), ("Round", 2)];
const NOTIFY_OPTIONS: &[(&str, &str, i32)] = &[
    ("All activity", "Everything that touches this workspace.", 0),
    ("Mentions only", "Direct mentions and shares.", 1),
    ("Nothing", "No desktop or email pings at all.", 2),
];
const DIGEST_OPTIONS: &[(&str, i32)] = &[("Never", 0), ("Daily", 1), ("Weekly", 2)];
const QUIET_FROM: &[(&str, i32)] = &[("20:00", 0), ("21:00", 1), ("22:00", 2), ("23:00", 3)];
const QUIET_UNTIL: &[(&str, i32)] = &[("06:00", 0), ("07:00", 1), ("08:00", 2), ("09:00", 3)];
const FONT_FAMILY: &[(&str, i32)] = &[("System UI", 0), ("Serif", 1), ("Monospace", 2)];
const TAB_SIZE: &[(&str, i32)] = &[("2", 0), ("4", 1), ("8", 2)];
const AUTOSAVE: &[(&str, i32)] = &[("5s", 0), ("15s", 1), ("30s", 2), ("1m", 3), ("5m", 4)];
const LANGUAGE: &[(&str, i32)] = &[("English", 0), ("Deutsch", 1), ("Español", 2), ("Français", 3)];
const CHANNEL: &[(&str, &str, i32)] = &[
    ("Stable", "Tested, safe, a few weeks behind.", 0),
    ("Beta", "New features, occasional bugs.", 1),
    ("Nightly", "Bleeding edge. Expect breakage.", 2),
];
const CACHE_LIMIT: &[(&str, i32)] = &[("256 MB", 0), ("1 GB", 1), ("4 GB", 2), ("Unlimited", 3)];

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Category { Appearance, Notifications, Editor, Account, Shortcuts, Advanced }

impl Category {
    fn label(self) -> &'static str {
        match self {
            Category::Appearance => "Appearance",
            Category::Notifications => "Notifications",
            Category::Editor => "Editor",
            Category::Account => "Account",
            Category::Shortcuts => "Keyboard shortcuts",
            Category::Advanced => "Advanced",
        }
    }
    fn description(self) -> &'static str {
        match self {
            Category::Appearance => "Theme, accent, density, and typography.",
            Category::Notifications => "What you get notified about, and when.",
            Category::Editor => "How the editor behaves while you write.",
            Category::Account => "Your profile and language.",
            Category::Shortcuts => "A reference. Filter by name.",
            Category::Advanced => "Developer options, updates, and data.",
        }
    }
    fn idx(self) -> usize {
        match self {
            Category::Appearance => 0, Category::Notifications => 1,
            Category::Editor => 2, Category::Account => 3,
            Category::Shortcuts => 4, Category::Advanced => 5,
        }
    }
}

const CATEGORIES: [Category; 6] = [
    Category::Appearance, Category::Notifications, Category::Editor,
    Category::Account, Category::Shortcuts, Category::Advanced,
];

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum SettingId {
    Theme, Accent, Density, FontSize, Radius, ReducedMotion,
    Desktop, Sound, NotifyAbout, Digest, QuietHours, QuietFrom, QuietUntil,
    FontFamily, TabSize, LineHeight, Spellcheck, AutoClose, Autosave,
    DisplayName, Email, Bio, Language,
    Developer, HardwareAccel, Channel, CacheLimit, MaxUndo,
}

impl SettingId {
    const ALL: [SettingId; 28] = [
        SettingId::Theme, SettingId::Accent, SettingId::Density,
        SettingId::FontSize, SettingId::Radius, SettingId::ReducedMotion,
        SettingId::Desktop, SettingId::Sound, SettingId::NotifyAbout,
        SettingId::Digest, SettingId::QuietHours, SettingId::QuietFrom, SettingId::QuietUntil,
        SettingId::FontFamily, SettingId::TabSize, SettingId::LineHeight,
        SettingId::Spellcheck, SettingId::AutoClose, SettingId::Autosave,
        SettingId::DisplayName, SettingId::Email, SettingId::Bio, SettingId::Language,
        SettingId::Developer, SettingId::HardwareAccel, SettingId::Channel,
        SettingId::CacheLimit, SettingId::MaxUndo,
    ];
    fn idx(self) -> usize {
        match self {
            SettingId::Theme => 0, SettingId::Accent => 1, SettingId::Density => 2,
            SettingId::FontSize => 3, SettingId::Radius => 4, SettingId::ReducedMotion => 5,
            SettingId::Desktop => 6, SettingId::Sound => 7, SettingId::NotifyAbout => 8,
            SettingId::Digest => 9, SettingId::QuietHours => 10, SettingId::QuietFrom => 11,
            SettingId::QuietUntil => 12,
            SettingId::FontFamily => 13, SettingId::TabSize => 14, SettingId::LineHeight => 15,
            SettingId::Spellcheck => 16, SettingId::AutoClose => 17, SettingId::Autosave => 18,
            SettingId::DisplayName => 19, SettingId::Email => 20, SettingId::Bio => 21,
            SettingId::Language => 22,
            SettingId::Developer => 23, SettingId::HardwareAccel => 24, SettingId::Channel => 25,
            SettingId::CacheLimit => 26, SettingId::MaxUndo => 27,
        }
    }
    fn category_of(self) -> Category {
        match self {
            SettingId::Theme | SettingId::Accent | SettingId::Density
            | SettingId::FontSize | SettingId::Radius | SettingId::ReducedMotion => Category::Appearance,
            SettingId::Desktop | SettingId::Sound | SettingId::NotifyAbout | SettingId::Digest
            | SettingId::QuietHours | SettingId::QuietFrom | SettingId::QuietUntil => Category::Notifications,
            SettingId::FontFamily | SettingId::TabSize | SettingId::LineHeight
            | SettingId::Spellcheck | SettingId::AutoClose | SettingId::Autosave => Category::Editor,
            SettingId::DisplayName | SettingId::Email | SettingId::Bio | SettingId::Language => Category::Account,
            SettingId::Developer | SettingId::HardwareAccel | SettingId::Channel
            | SettingId::CacheLimit | SettingId::MaxUndo => Category::Advanced,
        }
    }
    fn default_value(self) -> Value {
        match self {
            SettingId::Theme => Value::Int(0),
            SettingId::Accent => Value::Int(0),
            SettingId::Density => Value::Int(0),
            SettingId::FontSize => Value::Int(15),
            SettingId::Radius => Value::Int(1),
            SettingId::ReducedMotion => Value::Bool(false),
            SettingId::Desktop => Value::Bool(true),
            SettingId::Sound => Value::Bool(false),
            SettingId::NotifyAbout => Value::Int(1),
            SettingId::Digest => Value::Int(1),
            SettingId::QuietHours => Value::Bool(true),
            SettingId::QuietFrom => Value::Int(2),
            SettingId::QuietUntil => Value::Int(1),
            SettingId::FontFamily => Value::Int(0),
            SettingId::TabSize => Value::Int(1),
            SettingId::LineHeight => Value::Int(16),
            SettingId::Spellcheck => Value::Bool(true),
            SettingId::AutoClose => Value::Bool(true),
            SettingId::Autosave => Value::Int(2),
            SettingId::DisplayName => Value::Text("Alex Rowan".into()),
            SettingId::Email => Value::Text("alex@example.com".into()),
            SettingId::Bio => Value::Text("Notes, lists, and the occasional recipe.".into()),
            SettingId::Language => Value::Int(0),
            SettingId::Developer => Value::Bool(false),
            SettingId::HardwareAccel => Value::Bool(true),
            SettingId::Channel => Value::Int(0),
            SettingId::CacheLimit => Value::Int(1),
            SettingId::MaxUndo => Value::Text("200".into()),
        }
    }
    fn label(self) -> &'static str {
        match self {
            SettingId::Theme => "Theme",
            SettingId::Accent => "Accent colour",
            SettingId::Density => "Density",
            SettingId::FontSize => "Font size",
            SettingId::Radius => "Corner radius",
            SettingId::ReducedMotion => "Reduce motion",
            SettingId::Desktop => "Desktop notifications",
            SettingId::Sound => "Notification sound",
            SettingId::NotifyAbout => "Notify me about",
            SettingId::Digest => "Email digest",
            SettingId::QuietHours => "Quiet hours",
            SettingId::QuietFrom => "Quiet from",
            SettingId::QuietUntil => "Quiet until",
            SettingId::FontFamily => "Font family",
            SettingId::TabSize => "Tab size",
            SettingId::LineHeight => "Line height",
            SettingId::Spellcheck => "Spellcheck",
            SettingId::AutoClose => "Auto-close brackets",
            SettingId::Autosave => "Autosave interval",
            SettingId::DisplayName => "Display name",
            SettingId::Email => "Email",
            SettingId::Bio => "Bio",
            SettingId::Language => "Language",
            SettingId::Developer => "Developer mode",
            SettingId::HardwareAccel => "Hardware acceleration",
            SettingId::Channel => "Update channel",
            SettingId::CacheLimit => "Cache limit",
            SettingId::MaxUndo => "Max undo steps",
        }
    }
    fn description(self) -> &'static str {
        match self {
            SettingId::Theme => "The surface colour of the app.",
            SettingId::Accent => "Used for links, focus rings, and the primary button.",
            SettingId::Density => "How much padding each row of content gets.",
            SettingId::FontSize => "Base size for body text.",
            SettingId::Radius => "Roundness of cards and buttons.",
            SettingId::ReducedMotion => "Skip animations. Fades remain.",
            SettingId::Desktop => "Pop a system notification for mentions and shares.",
            SettingId::Sound => "Play a short tone with each notification.",
            SettingId::NotifyAbout => "Filter which events reach your device.",
            SettingId::Digest => "A summary of everything you missed.",
            SettingId::QuietHours => "Silence notifications during a window.",
            SettingId::QuietFrom => "Start of the quiet window.",
            SettingId::QuietUntil => "End of the quiet window.",
            SettingId::FontFamily => "What the editor renders in.",
            SettingId::TabSize => "Width of a tab character.",
            SettingId::LineHeight => "Vertical space between lines.",
            SettingId::Spellcheck => "Underline misspelled words.",
            SettingId::AutoClose => "Insert the closing bracket when you type the opener.",
            SettingId::Autosave => "How often edits are flushed to disk.",
            SettingId::DisplayName => "The name shown on your pages.",
            SettingId::Email => "Used for sign-in and notifications.",
            SettingId::Bio => "Shown on the profile card.",
            SettingId::Language => "Interface language.",
            SettingId::Developer => "Show the debug panel.",
            SettingId::HardwareAccel => "Use the GPU to draw. Turn off if you see tearing.",
            SettingId::Channel => "Which updates you receive.",
            SettingId::CacheLimit => "Upper bound on the local cache.",
            SettingId::MaxUndo => "How many undo steps to keep.",
        }
    }
}

#[derive(Clone, PartialEq)]
enum Value { Bool(bool), Int(i32), Text(String) }

#[derive(Clone, Copy)]
struct SettingsCtx { draft: RwSignal<Vec<Value>> }

fn default_settings() -> Vec<Value> {
    SettingId::ALL.iter().map(|id| id.default_value()).collect()
}

fn get_bool(v: &[Value], id: SettingId) -> bool {
    matches!(v.get(id.idx()), Some(Value::Bool(b)) if *b)
}
fn get_int(v: &[Value], id: SettingId) -> i32 {
    match v.get(id.idx()) { Some(Value::Int(i)) => *i, _ => 0 }
}
fn get_text(v: &[Value], id: SettingId) -> String {
    match v.get(id.idx()) { Some(Value::Text(s)) => s.clone(), _ => String::new() }
}
fn set_bool(v: &mut [Value], id: SettingId, b: bool) {
    if let Some(slot) = v.get_mut(id.idx()) { *slot = Value::Bool(b); }
}
fn set_int(v: &mut [Value], id: SettingId, i: i32) {
    if let Some(slot) = v.get_mut(id.idx()) { *slot = Value::Int(i); }
}
fn set_text(v: &mut [Value], id: SettingId, s: String) {
    if let Some(slot) = v.get_mut(id.idx()) { *slot = Value::Text(s); }
}

fn dirty_count(draft: &[Value], saved: &[Value]) -> usize {
    draft.iter().zip(saved.iter()).filter(|(a, b)| a != b).count()
}

fn category_dirty(draft: &[Value], saved: &[Value], cat: Category) -> bool {
    for id in SettingId::ALL.iter() {
        if id.category_of() != cat { continue; }
        if get_bool(draft, *id) != get_bool(saved, *id) { return true; }
        if get_int(draft, *id) != get_int(saved, *id) { return true; }
        if get_text(draft, *id) != get_text(saved, *id) { return true; }
    }
    false
}

fn scope_style(values: &[Value]) -> String {
    let theme = get_int(values, SettingId::Theme);
    let accent = get_int(values, SettingId::Accent);
    let font = get_int(values, SettingId::FontSize);
    let density = get_int(values, SettingId::Density);
    let radius = get_int(values, SettingId::Radius);
    let surface = match theme {
        0 => "var(--bg-deep)",
        1 => "var(--surface-2)",
        _ => "var(--bg)",
    };
    let border = if theme == 2 { "var(--border-strong)" } else { "var(--border)" };
    let pad = if density == 0 { "10px" } else { "4px" };
    let rad = match radius { 0 => "2px", 1 => "6px", _ => "14px" };
    let accent_color = match accent {
        0 => "var(--accent)",
        1 => "var(--fg-dim)",
        2 => "var(--warn)",
        3 => "var(--danger)",
        _ => "color-mix(in srgb, var(--accent) 45%, var(--fg))",
    };
    format!(
        "--st-accent: {}; --st-font: {}px; --st-pad: {}; --st-radius: {}; --st-surface: {}; --st-border: {};",
        accent_color, font, pad, rad, surface, border
    )
}

#[component]
pub fn SettingsModal() -> impl IntoView {
    let saved = RwSignal::new(default_settings());
    let open = RwSignal::new(false);
    let closing = RwSignal::new(false);

    let scope_saved = move || scope_style(&saved.get());
    let readout = move || {
        let s = saved.get();
        let theme = match get_int(&s, SettingId::Theme) { 0 => "Midnight", 1 => "Slate", _ => "Contrast" };
        let accent = match get_int(&s, SettingId::Accent) {
            0 => "Periwinkle", 1 => "Slate", 2 => "Amber", 3 => "Coral", _ => "Mist"
        };
        let font = get_int(&s, SettingId::FontSize);
        let density = match get_int(&s, SettingId::Density) { 0 => "Comfortable", _ => "Compact" };
        format!("{}, {}, {}px, {}", theme, accent, font, density)
    };

    view! {
        <div class="st-root">
            <div class="st-toolbar">
                <button class="btn btn-accent" on:click=move |_| open.set(true)>"Settings"</button>
                <span class="label">"Saved: "</span>
                <span class="st-readout label">{readout}</span>
            </div>
            <div class="mock-page st-scope st-app" style=scope_saved>
                <div class="mock-title">"Notes"</div>
                <div class="mock-block">"A quick note about the shape of the day. The light is already going."</div>
                <div class="mock-block">"Remember to pick up the parcel before six."</div>
                <div class="mock-block">"Try the new place on the corner this weekend."</div>
                <div class="st-app-footer">
                    <button class="btn st-app-primary">"Primary"</button>
                    <span class="st-app-chip">"Chip"</span>
                </div>
            </div>
            {move || if open.get() {
                Some(view! { <ModalContent saved=saved open=open closing=closing /> })
            } else { None }}
        </div>
    }
}

#[component]
fn ModalContent(
    saved: RwSignal<Vec<Value>>,
    open: RwSignal<bool>,
    closing: RwSignal<bool>,
) -> impl IntoView {
    let draft = RwSignal::new(saved.get_untracked());
    let category = RwSignal::new(Category::Appearance);
    let dir = RwSignal::new(1i32);
    let pulse_gen = RwSignal::new(0u64);
    let shortcut_filter = RwSignal::new(String::new());
    let saved_beat = RwSignal::new(false);
    let timers: RwSignal<Vec<TimeoutHandle>> = RwSignal::new(Vec::new());

    provide_context(SettingsCtx { draft });

    on_cleanup(move || {
        let list = timers.get_untracked();
        for h in list { h.clear(); }
    });

    let do_close = Callback::new(move |_: ()| {
        if closing.get_untracked() { return; }
        closing.set(true);
        let h = set_timeout_with_handle(
            move || { open.set(false); closing.set(false); },
            Duration::from_millis((CLOSE_MS + 40) as u64),
        );
        if let Ok(handle) = h { timers.update(|v| v.push(handle)); }
    });

    let try_close = Callback::new(move |_: ()| {
        let d = dirty_count(&draft.get_untracked(), &saved.get_untracked());
        if d > 0 { pulse_gen.update(|g| *g += 1); return; }
        do_close.run(());
    });

    let change_category = Callback::new(move |c: Category| {
        if category.get_untracked() == c { return; }
        let old_idx = category.get_untracked().idx();
        dir.set(if c.idx() > old_idx { 1 } else { -1 });
        category.set(c);
    });

    let cancel = move |_| {
        draft.set(saved.get_untracked());
        do_close.run(());
    };
    let save = move |_| {
        saved.set(draft.get_untracked());
        saved_beat.set(true);
        let h = set_timeout_with_handle(
            move || { saved_beat.set(false); do_close.run(()); },
            Duration::from_millis(SAVED_BEAT_MS as u64),
        );
        if let Ok(handle) = h { timers.update(|v| v.push(handle)); }
    };

    Effect::new(move |_| {
        let h = window_event_listener(ev::keydown, move |e: ev::KeyboardEvent| {
            if e.key() == "Escape" { try_close.run(()); }
        });
        on_cleanup(move || h.remove());
    });

    Effect::new(move |_| {
        set_timeout(move || {
            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                if let Ok(Some(el)) = doc.query_selector(".st-nav-btn") {
                    if let Ok(btn) = el.dyn_into::<web_sys::HtmlElement>() {
                        let _ = btn.focus();
                    }
                }
            }
        }, Duration::from_millis(0));
    });

    let nav_style = move || {
        let idx = category.get().idx();
        format!("--st-active: {}; --st-nav-h: {}px; --st-nav-gap: {}px;",
            idx, NAV_BTN_H, NAV_GAP)
    };
    let overlay_class = move || if closing.get() { "st-overlay closing" } else { "st-overlay" };
    let panel_class = move || if closing.get() { "st-panel closing" } else { "st-panel" };

    let on_overlay_click = move |ev: ev::MouseEvent| {
        if let Some(t) = ev.target() {
            if let Ok(el) = t.dyn_into::<web_sys::Element>() {
                if el.class_list().contains("st-overlay") { try_close.run(()); }
            }
        }
    };

    let dirty = move || dirty_count(&draft.get(), &saved.get());
    let chip_view = move || {
        let n = dirty_count(&draft.get(), &saved.get());
        if n == 0 {
            return view! { <span class="st-chip-clean label">"All changes saved"</span> }.into_any();
        }
        let g = pulse_gen.get();
        let text = format!("{} unsaved change{}", n, if n == 1 { "" } else { "s" });
        view! {
            <For
                each=move || vec![g]
                key=|x| *x
                children=move |_| {
                    let text = text.clone();
                    view! {
                        <span class="st-chip-dirty label">
                            <span class="st-chip-dot"></span>{text}
                        </span>
                    }.into_any()
                }
            />
        }.into_any()
    };

    let save_btn_disabled = move || dirty() == 0 || saved_beat.get();
    let save_btn_text = move || if saved_beat.get() { "\u{2713} Saved" } else { "Save changes" };

    let pane_view = move || {
        let cat = category.get();
        let d = dir.get();
        view! {
            <For
                each=move || vec![cat]
                key=|c| *c
                children=move |c: Category| {
                    let pane: AnyView = match c {
                        Category::Appearance => render_appearance().into_any(),
                        Category::Notifications => render_notifications().into_any(),
                        Category::Editor => render_editor().into_any(),
                        Category::Account => render_account().into_any(),
                        Category::Shortcuts => render_shortcuts(shortcut_filter).into_any(),
                        Category::Advanced => render_advanced().into_any(),
                    };
                    view! {
                        <div class="st-pane" style=format!("--st-dir: {}", d)>
                            {pane}
                        </div>
                    }.into_any()
                }
            />
        }.into_any()
    };

    view! {
        <div class=overlay_class on:click=on_overlay_click>
            <div class=panel_class role="dialog" aria-modal="true">
                <nav class="st-nav" style=nav_style>
                    <div class="st-nav-inner">
                        <div class="st-nav-indicator"></div>
                        {CATEGORIES.iter().map(|&c| {
                            let is_active = move || category.get() == c;
                            let has_dot = move || category_dirty(&draft.get(), &saved.get(), c);
                            let onclick = move |_| change_category.run(c);
                            view! {
                                <button class="st-nav-btn" class:active=is_active on:click=onclick>
                                    <span>{c.label()}</span>
                                    {move || if has_dot() {
                                        view! { <span class="st-nav-dot"></span> }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    }}
                                </button>
                            }
                        }).collect_view()}
                    </div>
                </nav>
                <div class="st-main">
                    <header class="st-head">
                        <div class="st-head-text">
                            <h2 class="st-title">{move || category.get().label()}</h2>
                            <p class="st-subtitle">{move || category.get().description()}</p>
                        </div>
                        <button class="st-x" aria-label="Close" on:click=move |_| try_close.run(())>"\u{00d7}"</button>
                    </header>
                    <div class="st-scroll">{pane_view}</div>
                    <footer class="st-foot">
                        {chip_view}
                        <div class="st-foot-actions">
                            <button class="btn" on:click=cancel>"Cancel"</button>
                            <button class="btn btn-accent" disabled=save_btn_disabled on:click=save>
                                {save_btn_text}
                            </button>
                        </div>
                    </footer>
                </div>
            </div>
        </div>
    }
}

fn render_appearance() -> impl IntoView {
    let ctx = expect_context::<SettingsCtx>();
    let draft = ctx.draft;
    let scope_draft = move || scope_style(&draft.get());
    view! {
        <div>
            <div class="st-preview">
                <div class="mock-page st-scope" style=scope_draft>
                    <div class="mock-title">"Preview"</div>
                    <div class="mock-block">"Live preview of the draft. Save morphs the backdrop."</div>
                    <div class="mock-block">"Accent, radius, density and font size update as you change them."</div>
                    <div class="st-app-footer">
                        <button class="btn st-app-primary">"Primary"</button>
                        <span class="st-app-chip">"Chip"</span>
                    </div>
                </div>
            </div>
            <SegRow id=SettingId::Theme options=THEME_OPTIONS />
            <SwatchesRow id=SettingId::Accent options=ACCENT_OPTIONS />
            <RadioRow id=SettingId::Density options=DENSITY_OPTIONS />
            <SliderRow id=SettingId::FontSize min=12 max=20 step=1 suffix="px" />
            <SegRow id=SettingId::Radius options=RADIUS_OPTIONS />
            <ToggleRow id=SettingId::ReducedMotion />
        </div>
    }
}

fn render_notifications() -> impl IntoView {
    view! {
        <div>
            <ToggleRow id=SettingId::Desktop />
            <ToggleRow id=SettingId::Sound />
            <RadioRow id=SettingId::NotifyAbout options=NOTIFY_OPTIONS />
            <DropdownRow id=SettingId::Digest options=DIGEST_OPTIONS />
            <ToggleRow id=SettingId::QuietHours />
            {move || {
                let ctx = expect_context::<SettingsCtx>();
                let on = get_bool(&ctx.draft.get(), SettingId::QuietHours);
                if on {
                    view! {
                        <div class="st-row-pair">
                            <DropdownRow id=SettingId::QuietFrom options=QUIET_FROM />
                            <DropdownRow id=SettingId::QuietUntil options=QUIET_UNTIL />
                        </div>
                    }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }
            }}
        </div>
    }
}

fn render_editor() -> impl IntoView {
    view! {
        <div>
            <DropdownRow id=SettingId::FontFamily options=FONT_FAMILY />
            <SegRow id=SettingId::TabSize options=TAB_SIZE />
            <SliderRow id=SettingId::LineHeight min=12 max=20 step=1 suffix=" (1.x)" />
            <ToggleRow id=SettingId::Spellcheck />
            <ToggleRow id=SettingId::AutoClose />
            <DropdownRow id=SettingId::Autosave options=AUTOSAVE />
        </div>
    }
}

fn render_account() -> impl IntoView {
    view! {
        <div>
            <TextRow id=SettingId::DisplayName input_type="text" />
            <TextRow id=SettingId::Email input_type="email" />
            <TextareaRow id=SettingId::Bio />
            <DropdownRow id=SettingId::Language options=LANGUAGE />
            <div class="st-row st-row-danger-actions">
                <div class="st-row-left">
                    <div class="st-row-title">"Account actions"</div>
                    <div class="st-row-desc">"Placeholders. No handler wired."</div>
                </div>
                <div class="st-row-actions">
                    <button class="btn">"Sign out of other devices"</button>
                    <button class="btn st-danger-btn">"Delete account"</button>
                </div>
            </div>
        </div>
    }
}

fn render_shortcuts(shortcut_filter: RwSignal<String>) -> impl IntoView {
    view! {
        <div>
            <div class="st-row st-row-filter">
                <div class="st-row-left">
                    <div class="st-row-title">"Filter"</div>
                    <div class="st-row-desc">"Type to narrow the list below."</div>
                </div>
                <input
                    class="st-input"
                    type="text"
                    placeholder="Filter shortcuts…"
                    prop:value=move || shortcut_filter.get()
                    on:input=move |ev| {
                        if let Some(t) = ev.target() {
                            if let Ok(inp) = t.dyn_into::<web_sys::HtmlInputElement>() {
                                shortcut_filter.set(inp.value());
                            }
                        }
                    }
                />
            </div>
            <div class="st-shortcut-list">
                {move || {
                    let f = shortcut_filter.get().to_lowercase();
                    SHORTCUTS.iter()
                        .filter(|(label, _)| f.is_empty() || label.to_lowercase().contains(&f))
                        .map(|(label, key)| {
                            view! {
                                <div class="st-shortcut-row">
                                    <span class="st-shortcut-desc">{*label}</span>
                                    <kbd class="st-kbd">{*key}</kbd>
                                </div>
                            }.into_any()
                        })
                        .collect_view()
                }}
            </div>
            <div class="st-row">
                <div class="st-row-left">
                    <div class="st-row-title">"Restore defaults"</div>
                    <div class="st-row-desc">"Reset every shortcut to the factory set."</div>
                </div>
                <button class="btn">"Restore"</button>
            </div>
        </div>
    }
}

fn render_advanced() -> impl IntoView {
    let ctx = expect_context::<SettingsCtx>();
    let draft = ctx.draft;
    let reset_all = move |_| {
        draft.update(|v| *v = default_settings());
    };
    view! {
        <div>
            <ToggleRow id=SettingId::Developer />
            <ToggleRow id=SettingId::HardwareAccel />
            <RadioRow id=SettingId::Channel options=CHANNEL />
            <DropdownRow id=SettingId::CacheLimit options=CACHE_LIMIT />
            <TextRow id=SettingId::MaxUndo input_type="number" />
            <div class="st-row">
                <div class="st-row-left">
                    <div class="st-row-title">"Data folder"</div>
                    <div class="st-row-desc">"Where local notes live on this device."</div>
                </div>
                <span class="st-mono">"~/.local/share/feel_labs"</span>
            </div>
            <div class="st-row">
                <div class="st-row-left">
                    <div class="st-row-title">"Reset all settings"</div>
                    <div class="st-row-desc">"Sets the draft to defaults. Still needs Save."</div>
                </div>
                <button class="btn st-danger-btn" on:click=reset_all>"Reset"</button>
            </div>
        </div>
    }
}

// ---------- Row helper components ----------

#[component]
fn ToggleRow(id: SettingId) -> impl IntoView {
    let ctx = expect_context::<SettingsCtx>();
    let draft = ctx.draft;
    let on = move || get_bool(&draft.get(), id);
    let toggle = move |_| {
        draft.update(|v| {
            let cur = get_bool(v, id);
            set_bool(v, id, !cur);
        });
    };
    view! {
        <div class="st-row">
            <div class="st-row-left">
                <div class="st-row-title">{id.label()}</div>
                <div class="st-row-desc">{id.description()}</div>
            </div>
            <button class="st-toggle" class:on=on role="switch"
                aria-checked=move || if on() { "true" } else { "false" }
                aria-label=id.label()
                on:click=toggle>
                <span class="st-toggle-knob"></span>
            </button>
        </div>
    }
}

#[component]
fn SliderRow(id: SettingId, min: i32, max: i32, step: i32, suffix: &'static str) -> impl IntoView {
    let ctx = expect_context::<SettingsCtx>();
    let draft = ctx.draft;
    let v = move || get_int(&draft.get(), id);
    let on = move |ev: ev::Event| {
        if let Some(t) = ev.target() {
            if let Ok(inp) = t.dyn_into::<web_sys::HtmlInputElement>() {
                if let Ok(n) = inp.value().parse::<i32>() {
                    draft.update(|list| set_int(list, id, n));
                }
            }
        }
    };
    let readout = move || format!("{}{}", v(), suffix);
    view! {
        <div class="st-row">
            <div class="st-row-left">
                <div class="st-row-title">{id.label()}</div>
                <div class="st-row-desc">{id.description()}</div>
            </div>
            <div class="st-slider-row">
                <input type="range" class="st-slider" min=min max=max step=step
                    prop:value=move || v().to_string()
                    on:input=on />
                <span class="st-slider-val">{readout}</span>
            </div>
        </div>
    }
}

#[component]
fn DropdownRow(id: SettingId, options: &'static [(&'static str, i32)]) -> impl IntoView {
    let ctx = expect_context::<SettingsCtx>();
    let draft = ctx.draft;
    let cur = move || get_int(&draft.get(), id);
    let on = move |ev: ev::Event| {
        if let Some(t) = ev.target() {
            if let Ok(sel) = t.dyn_into::<web_sys::HtmlSelectElement>() {
                if let Ok(n) = sel.value().parse::<i32>() {
                    draft.update(|list| set_int(list, id, n));
                }
            }
        }
    };
    view! {
        <div class="st-row">
            <div class="st-row-left">
                <div class="st-row-title">{id.label()}</div>
                <div class="st-row-desc">{id.description()}</div>
            </div>
            <div class="st-select-wrap">
                <select class="st-select" on:change=on>
                    {options.iter().map(|(label, val)| {
                        let is_sel = cur() == *val;
                        view! {
                            <option value=val.to_string() selected=is_sel>{*label}</option>
                        }
                    }).collect_view()}
                </select>
            </div>
        </div>
    }
}

#[component]
fn RadioRow(id: SettingId, options: &'static [(&'static str, &'static str, i32)]) -> impl IntoView {
    let ctx = expect_context::<SettingsCtx>();
    let draft = ctx.draft;
    let cur = move || get_int(&draft.get(), id);
    view! {
        <div class="st-row st-row-stack">
            <div class="st-row-left">
                <div class="st-row-title">{id.label()}</div>
                <div class="st-row-desc">{id.description()}</div>
            </div>
            <div class="st-radios">
                {options.iter().map(|(label, desc, val)| {
                    let v = *val;
                    let is_sel = move || cur() == v;
                    let onclick = move |_| {
                        draft.update(|list| set_int(list, id, v));
                    };
                    view! {
                        <button class="st-radio" class:on=is_sel role="radio"
                            aria-checked=move || if is_sel() { "true" } else { "false" }
                            on:click=onclick>
                            <span class="st-radio-title">{*label}</span>
                            <span class="st-radio-desc">{*desc}</span>
                        </button>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

#[component]
fn SegRow(id: SettingId, options: &'static [(&'static str, i32)]) -> impl IntoView {
    let ctx = expect_context::<SettingsCtx>();
    let draft = ctx.draft;
    let cur = move || get_int(&draft.get(), id);
    view! {
        <div class="st-row">
            <div class="st-row-left">
                <div class="st-row-title">{id.label()}</div>
                <div class="st-row-desc">{id.description()}</div>
            </div>
            <div class="seg">
                {options.iter().map(|(label, val)| {
                    let v = *val;
                    let is_sel = move || cur() == v;
                    let onclick = move |_| { draft.update(|list| set_int(list, id, v)); };
                    view! {
                        <button class="seg-btn" class:active=is_sel on:click=onclick>
                            {*label}
                        </button>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

#[component]
fn SwatchesRow(id: SettingId, options: &'static [(&'static str, &'static str)]) -> impl IntoView {
    let ctx = expect_context::<SettingsCtx>();
    let draft = ctx.draft;
    let cur = move || get_int(&draft.get(), id);
    view! {
        <div class="st-row">
            <div class="st-row-left">
                <div class="st-row-title">{id.label()}</div>
                <div class="st-row-desc">{id.description()}</div>
            </div>
            <div class="st-swatches">
                {options.iter().enumerate().map(|(i, (label, color))| {
                    let idx = i as i32;
                    let is_sel = move || cur() == idx;
                    let onclick = move |_| {
                        draft.update(|list| set_int(list, id, idx));
                    };
                    let style = format!("background: {}", color);
                    view! {
                        <button class="st-swatch" class:active=is_sel aria-label=*label
                            style=style on:click=onclick></button>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

#[component]
fn TextRow(id: SettingId, input_type: &'static str) -> impl IntoView {
    let ctx = expect_context::<SettingsCtx>();
    let draft = ctx.draft;
    let v = move || get_text(&draft.get(), id);
    let on = move |ev: ev::Event| {
        if let Some(t) = ev.target() {
            if let Ok(inp) = t.dyn_into::<web_sys::HtmlInputElement>() {
                draft.update(|list| set_text(list, id, inp.value()));
            }
        }
    };
    view! {
        <div class="st-row st-row-stack">
            <div class="st-row-left">
                <div class="st-row-title">{id.label()}</div>
                <div class="st-row-desc">{id.description()}</div>
            </div>
            <input class="st-input" type=input_type prop:value=v on:input=on />
        </div>
    }
}

#[component]
fn TextareaRow(id: SettingId) -> impl IntoView {
    let ctx = expect_context::<SettingsCtx>();
    let draft = ctx.draft;
    let v = move || get_text(&draft.get(), id);
    let on = move |ev: ev::Event| {
        if let Some(t) = ev.target() {
            if let Ok(ta) = t.dyn_into::<web_sys::HtmlTextAreaElement>() {
                draft.update(|list| set_text(list, id, ta.value()));
            }
        }
    };
    view! {
        <div class="st-row st-row-stack">
            <div class="st-row-left">
                <div class="st-row-title">{id.label()}</div>
                <div class="st-row-desc">{id.description()}</div>
            </div>
            <textarea class="st-textarea" rows="3" prop:value=v on:input=on></textarea>
        </div>
    }
}