use leptos::prelude::*;

const ROWS: &[&str] = &[
    "Weekly planning",
    "Reading list",
    "Trip itinerary",
    "Recipes to try",
    "Meeting notes",
    "Budget 2026",
];

const HOVER_EFFECTS: &[(&str, &str)] = &[
    ("underline", "Underline draw"),
    ("nudge", "Nudge + pill"),
    ("wave", "Letter wave"),
];

const WIGGLE_DEG: i32 = 14;
const WIGGLE_MS: u32 = 520;
const EYE_MS: u32 = 480;
const LETTER_STAGGER_MS: u32 = 18;

#[derive(Clone, Copy)]
struct RowState {
    id: usize,
    pen_gen: RwSignal<u64>,
    disabled: RwSignal<bool>,
}

#[component]
pub fn MainMenu() -> impl IntoView {
    let root_style = format!(
        "--mm-wiggle-deg: {}deg; --mm-wiggle-ms: {}ms; --mm-eye-ms: {}ms; --mm-letter-stagger: {}ms;",
        WIGGLE_DEG, WIGGLE_MS, EYE_MS, LETTER_STAGGER_MS,
    );

    let columns = HOVER_EFFECTS
        .iter()
        .map(|&(kind, label)| {
            view! { <MenuColumn kind=kind label=label /> }
        })
        .collect_view();

    view! {
        <div class="mm-root" style=root_style>
            <div class="mm-columns">{columns}</div>
        </div>
    }
}

#[component]
fn MenuColumn(kind: &'static str, label: &'static str) -> impl IntoView {
    let rows = RwSignal::new(
        (0..ROWS.len())
            .map(|id| RowState {
                id,
                pen_gen: RwSignal::new(0u64),
                disabled: RwSignal::new(false),
            })
            .collect::<Vec<_>>(),
    );

    view! {
        <div class="mm-column" data-hover=kind>
            <div class="mm-column-head">
                <span class="label">"Pages"</span>
                <span class="mm-effect-name">{label}</span>
            </div>
            <div class="mm-rows">
                <For
                    each=move || rows.get()
                    key=|r| r.id
                    children=move |row: RowState| {
                        let title = ROWS[row.id];
                        let pen_gen = row.pen_gen;
                        let disabled = row.disabled;
                        let title_view = render_title(title, kind);
                        let row_class = move || {
                            if disabled.get() { "mm-row disabled" } else { "mm-row" }
                        };
                        let on_pen = move |_| {
                            pen_gen.update(|g| *g = g.wrapping_add(1));
                        };
                        let on_eye = move |_| {
                            disabled.update(|d| *d = !*d);
                        };
                        view! {
                            <div class=row_class>
                                <span class="mm-title">{title_view}</span>
                                <div class="mm-icons">
                                    <button
                                        class="mm-icon-btn"
                                        aria-label="Edit title"
                                        on:click=on_pen
                                    >
                                        <For
                                            each=move || vec![pen_gen.get()]
                                            key=|g| *g
                                            children=move |g| {
                                                let pen_cls = if g > 0 { "mm-pen mm-pen-wiggle" } else { "mm-pen" };
                                                view! {
                                                    <svg class="mm-icon-svg mm-pen-svg" viewBox="0 0 24 24" width="20" height="20">
                                                        <path d="M12 20h9" />
                                                        <path class=pen_cls d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z" />
                                                    </svg>
                                                }
                                            }
                                        />
                                    </button>
                                    <button
                                        class="mm-icon-btn"
                                        aria-label=move || if disabled.get() { "Enable page" } else { "Disable page" }
                                        on:click=on_eye
                                    >
                                        <svg class="mm-icon-svg mm-eye" viewBox="0 0 24 24" width="20" height="20">
                                            <path class="mm-eye-upper" d="M1 12s4-8 11-8 11 8 11 8" />
                                            <path class="mm-eye-lower" d="M1 12s4 8 11 8 11-8 11-8" />
                                            <circle class="mm-eye-pupil" cx="12" cy="12" r="3" />
                                        </svg>
                                    </button>
                                </div>
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}

fn render_title(title: &'static str, kind: &'static str) -> AnyView {
    if kind == "wave" {
        let chars: Vec<(usize, char)> = title.chars().enumerate().collect();
        view! {
            <span class="mm-title-wave">
                {chars.into_iter().map(|(i, c)| {
                    let style = format!("--i: {}", i);
                    view! { <span class="mm-letter" style=style>{c.to_string()}</span> }
                }).collect_view()}
            </span>
        }
        .into_any()
    } else {
        view! { <span>{title}</span> }.into_any()
    }
}