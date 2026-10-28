use leptos::ev;
use leptos::html;
use leptos::prelude::*;
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use wasm_bindgen::JsCast;

const DEBOUNCE_MS: u64 = 140;
const DEMO_TICK_MS: u64 = 85;
const DEMO_HOLD_TICKS: u32 = 16;
const EXIT_MS: u32 = 340;
const EXIT_MARGIN_MS: u32 = 30;
const EXIT_LEAD_MS: u32 = 120;
const STAGGER_MS: u32 = 36;

struct Doc {
    title: &'static str,
    breadcrumb: &'static str,
    snippet: &'static str,
    keywords: &'static [&'static str],
}

const DOCS: &[Doc] = &[
    Doc { title: "Budget 2026", breadcrumb: "Finance > Budget",
        snippet: "Move the streaming and gym subscriptions to annual billing, saves about 14% over twelve months.",
        keywords: &["money", "subscriptions", "annual"] },
    Doc { title: "Trip itinerary", breadcrumb: "Travel > Itinerary",
        snippet: "Three days walking, one day slow. Book the early train so the afternoon is open for the museum.",
        keywords: &["travel", "train", "museum"] },
    Doc { title: "Meeting notes", breadcrumb: "Work > Notes",
        snippet: "Agreed to ship the smaller refactor first. Riskiest part is the schema migration; second pass next Thursday.",
        keywords: &["work", "ship", "refactor", "schema"] },
    Doc { title: "Reading list", breadcrumb: "Library > Want to read",
        snippet: "Half-finished: the essay collection. Next: the one about maps. Skip the airport novel, it didn't grab you.",
        keywords: &["books", "essay", "novel", "read"] },
    Doc { title: "Recipes to try", breadcrumb: "Kitchen > Untested",
        snippet: "Lentil soup with charred lemon. Salt early, bloom the spices in oil, finish with yogurt and mint.",
        keywords: &["food", "soup", "lemon", "cook"] },
    Doc { title: "Workout log", breadcrumb: "Training > Log",
        snippet: "Squats felt heavy this week. Pull volume back 10 percent, add a second long walk instead.",
        keywords: &["training", "squats", "walk", "volume"] },
    Doc { title: "Project retro", breadcrumb: "Work > Retro Q3",
        snippet: "What worked: short written updates, no status meetings. What didn't: the shared checklist nobody updated.",
        keywords: &["work", "retro", "updates", "checklist", "notes"] },
    Doc { title: "Gift ideas", breadcrumb: "Notes > Gifts",
        snippet: "A good pen, a warm scarf, or the cookbook she keeps bringing up. Skip anything with a screen.",
        keywords: &["presents", "pen", "scarf", "cookbook"] },
    Doc { title: "Apartment search", breadcrumb: "Home > Search",
        snippet: "Top floor, south light, in-unit laundry. Neighborhood feel matters more than exact square meters.",
        keywords: &["home", "light", "laundry", "neighborhood"] },
    Doc { title: "Book notes", breadcrumb: "Library > Notes",
        snippet: "The chapter on habits argues that friction is a design choice. Worth borrowing for the writing app.",
        keywords: &["books", "habits", "friction", "design"] },
    Doc { title: "Password rotation checklist", breadcrumb: "Security > Rotation",
        snippet: "Bank, email, cloud backups. Two-factor on each. The recovery codes go in the safe, not the notes app.",
        keywords: &["security", "passwords", "two-factor", "recovery"] },
    Doc { title: "Weekly review", breadcrumb: "Notes > Weekly",
        snippet: "Three things done, two pushed, one dropped on purpose. Next week: shorter list, fewer meetings.",
        keywords: &["review", "weekly", "list", "meetings", "notes"] },
    Doc { title: "Journal", breadcrumb: "Notes > Journal",
        snippet: "Morning pages, three days in a row. Slightly less scattered. Note to keep the pen and notebook on the desk.",
        keywords: &["notes", "journal", "morning pages"] },
    Doc { title: "Sleep log", breadcrumb: "Notes > Sleep",
        snippet: "In bed at 22:40, up at 06:15. Woke twice. Try reading instead of phone in bed.",
        keywords: &["notes", "sleep", "log"] },
];

const DEMO_QUERIES: &[&str] = &["trip", "budget october", "reading list", "notes"];

#[derive(Clone, Copy)]
struct Hit { idx: usize, score: i32 }

#[derive(Clone, Copy, PartialEq)]
enum DemoPhase { Typing, Holding, Erasing }

#[derive(Clone, Copy)]
struct DemoState {
    query_idx: usize,
    phase: DemoPhase,
    char_pos: usize,
    ticks_left: u32,
}

#[derive(Clone)]
struct DisplayRow {
    idx: usize,
    leaving: RwSignal<bool>,
    leave_gen: u64,
    enter_i: usize,
    enter_delay_ms: u32,
}

fn compute_results(query: &str) -> Vec<Hit> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return (0..DOCS.len().min(4)).map(|idx| Hit { idx, score: 0 }).collect();
    }
    let lower = trimmed.to_ascii_lowercase();
    let tokens: Vec<&str> = lower.split_whitespace().filter(|t| !t.is_empty()).collect();
    if tokens.is_empty() {
        return (0..DOCS.len().min(4)).map(|idx| Hit { idx, score: 0 }).collect();
    }
    let mut hits: Vec<Hit> = DOCS.iter().enumerate().filter_map(|(idx, doc)| {
        let title_l = doc.title.to_ascii_lowercase();
        let snippet_l = doc.snippet.to_ascii_lowercase();
        let mut score = 0i32;
        for t in &tokens {
            if title_l.contains(t) { score += 3; }
            if doc.keywords.iter().any(|k| k.to_ascii_lowercase().contains(t)) { score += 2; }
            if snippet_l.contains(t) { score += 1; }
        }
        if score > 0 { Some(Hit { idx, score }) } else { None }
    }).collect();
    hits.sort_by(|a, b| b.score.cmp(&a.score));
    hits
}

fn highlight_segments(snippet: &str, query: &str) -> Vec<(String, bool)> {
    if query.trim().is_empty() || snippet.is_empty() { return vec![(snippet.to_string(), false)]; }
    let lower_snippet = snippet.to_ascii_lowercase();
    let lower_query = query.to_ascii_lowercase();
    let tokens: Vec<&str> = lower_query.split_whitespace().filter(|t| !t.is_empty()).collect();
    if tokens.is_empty() { return vec![(snippet.to_string(), false)]; }
    let mut marked = vec![false; snippet.len()];
    for t in &tokens {
        let mut from = 0usize;
        while from < lower_snippet.len() {
            match lower_snippet[from..].find(t) {
                Some(rel) => {
                    let a = from + rel;
                    let b = a + t.len();
                    for i in a..b.min(marked.len()) { marked[i] = true; }
                    from = b;
                }
                None => break,
            }
        }
    }
    let mut segments: Vec<(String, bool)> = Vec::new();
    let mut start = 0usize;
    let mut cur = marked.first().copied().unwrap_or(false);
    for (byte_idx, _c) in snippet.char_indices().skip(1) {
        let m = marked.get(byte_idx).copied().unwrap_or(false);
        if m != cur {
            segments.push((snippet[start..byte_idx].to_string(), cur));
            start = byte_idx;
            cur = m;
        }
    }
    if start < snippet.len() { segments.push((snippet[start..].to_string(), cur)); }
    segments
}

fn demo_tick(demo: RwSignal<DemoState>, query: RwSignal<String>) {
    demo.update(|s| {
        let q = DEMO_QUERIES[s.query_idx % DEMO_QUERIES.len()];
        let chars: Vec<char> = q.chars().collect();
        match s.phase {
            DemoPhase::Typing => {
                if s.char_pos < chars.len() {
                    s.char_pos += 1;
                    query.set(chars[..s.char_pos].iter().collect());
                } else {
                    s.phase = DemoPhase::Holding;
                    s.ticks_left = DEMO_HOLD_TICKS;
                }
            }
            DemoPhase::Holding => {
                if s.ticks_left == 0 { s.phase = DemoPhase::Erasing; } else { s.ticks_left -= 1; }
            }
            DemoPhase::Erasing => {
                if s.char_pos > 0 {
                    s.char_pos -= 1;
                    query.set(chars[..s.char_pos].iter().collect());
                } else {
                    s.query_idx = (s.query_idx + 1) % DEMO_QUERIES.len();
                    s.phase = DemoPhase::Typing;
                }
            }
        }
    });
}

fn prefers_reduced_motion() -> bool {
    let Some(win) = web_sys::window() else { return false; };
    match win.match_media("(prefers-reduced-motion: reduce)") {
        Ok(Some(m)) => m.matches(),
        Ok(None) => false,
        Err(_) => false,
    }
}

fn initial_display() -> Vec<DisplayRow> {
    (0..DOCS.len().min(4)).enumerate().map(|(i, idx)| DisplayRow {
        idx,
        leaving: RwSignal::new(false),
        leave_gen: 0,
        enter_i: i,
        enter_delay_ms: 0,
    }).collect()
}

fn commit_results(
    new_hits: Vec<Hit>,
    query: String,
    results: RwSignal<Vec<DisplayRow>>,
    committed_query: RwSignal<String>,
    commit_gen: u64,
    timeouts: RwSignal<Vec<TimeoutHandle>>,
    body_ref: NodeRef<html::Div>,
    reduced: bool,
) {
    let old = results.get_untracked();
    let new_idx_set: HashSet<usize> = new_hits.iter().map(|h| h.idx).collect();

    let has_new_leaving = !reduced && old.iter().any(|r|
        !new_idx_set.contains(&r.idx) && !r.leaving.get_untracked()
    );
    let enter_delay_ms = if has_new_leaving { EXIT_LEAD_MS } else { 0 };

    let old_order: Vec<usize> = old.iter().map(|r| r.idx).collect();
    let mut old_by_idx: HashMap<usize, DisplayRow> = old.into_iter().map(|r| (r.idx, r)).collect();

    let mut new_list: Vec<DisplayRow> = Vec::new();
    let mut enter_counter = 0usize;
    for hit in &new_hits {
        if let Some(r) = old_by_idx.remove(&hit.idx) {
            if r.leaving.get_untracked() { r.leaving.set(false); }
            new_list.push(r);
        } else {
            new_list.push(DisplayRow {
                idx: hit.idx,
                leaving: RwSignal::new(false),
                leave_gen: 0,
                enter_i: enter_counter,
                enter_delay_ms,
            });
            enter_counter += 1;
        }
    }

    if reduced {
        results.set(new_list);
        committed_query.set(query);
        if let Some(el) = body_ref.get_untracked() { el.set_scroll_top(0); }
        return;
    }

    let mut insert_at = 0usize;
    for old_idx in old_order {
        if new_idx_set.contains(&old_idx) {
            if let Some(pos) = new_list.iter().position(|r| r.idx == old_idx) {
                insert_at = pos + 1;
            }
        } else if let Some(mut row) = old_by_idx.remove(&old_idx) {
            if !row.leaving.get_untracked() {
                row.leaving.set(true);
                row.leave_gen = commit_gen;
            }
            new_list.insert(insert_at, row);
            insert_at += 1;
        }
    }
    results.set(new_list);

    if has_new_leaving {
        let h = set_timeout_with_handle(
            move || {
                results.update(|v| v.retain(|r|
                    !(r.leaving.get_untracked() && r.leave_gen == commit_gen)
                ));
            },
            Duration::from_millis((EXIT_MS + EXIT_MARGIN_MS) as u64),
        );
        if let Ok(handle) = h { timeouts.update(|v| v.push(handle)); }
    }

    committed_query.set(query);
    if let Some(el) = body_ref.get_untracked() { el.set_scroll_top(0); }
}

#[component]
pub fn SearchBar() -> impl IntoView {
    let reduced = prefers_reduced_motion();
    let query = RwSignal::new(String::new());
    let committed_query = RwSignal::new(String::new());
    let results: RwSignal<Vec<DisplayRow>> = RwSignal::new(initial_display());
    let searching = RwSignal::new(false);
    let debounce_handle: RwSignal<Option<TimeoutHandle>> = RwSignal::new(None);
    let demo_running = RwSignal::new(true);
    let demo_state = RwSignal::new(DemoState {
        query_idx: 0, phase: DemoPhase::Typing, char_pos: 0, ticks_left: 0,
    });
    let commit_gen = RwSignal::new(1u64);
    let timeouts: RwSignal<Vec<TimeoutHandle>> = RwSignal::new(Vec::new());

    let input_ref = NodeRef::<html::Input>::new();
    let body_ref = NodeRef::<html::Div>::new();

    Effect::new(move |_| {
        let q = query.get();
        if let Some(el) = input_ref.get() {
            if el.value() != q { el.set_value(&q); }
        }
    });

    Effect::new(move |_| {
        let _ = query.get();
        searching.set(true);
        if let Some(h) = debounce_handle.get_untracked() { h.clear(); }
        let h = set_timeout_with_handle(
            move || {
                let q = query.get_untracked();
                let hits = compute_results(&q);
                let gen_id = commit_gen.get_untracked();
                commit_gen.set(gen_id + 1);
                commit_results(hits, q, results, committed_query, gen_id, timeouts, body_ref, reduced);
                searching.set(false);
            },
            Duration::from_millis(DEBOUNCE_MS),
        );
        match h {
            Ok(handle) => debounce_handle.set(Some(handle)),
            Err(e) => leptos::logging::error!("search_bar: set timeout: {e:?}"),
        }
    });

    Effect::new(move |_| {
        let h = set_interval_with_handle(
            move || {
                if !demo_running.get_untracked() { return; }
                demo_tick(demo_state, query);
            },
            Duration::from_millis(DEMO_TICK_MS),
        );
        match h {
            Ok(handle) => on_cleanup(move || handle.clear()),
            Err(e) => leptos::logging::error!("search_bar: set interval: {e:?}"),
        }
    });

    on_cleanup(move || {
        if let Some(h) = debounce_handle.get_untracked() { h.clear(); }
        let list = timeouts.get_untracked();
        for h in list { h.clear(); }
    });

    let on_input = move |ev: ev::Event| {
        let Some(target) = ev.target() else { return; };
        let Ok(inp) = target.dyn_into::<web_sys::HtmlInputElement>() else { return; };
        demo_running.set(false);
        query.set(inp.value());
    };
    let on_focus = move |_| { demo_running.set(false); };
    let clear = move |_| {
        demo_running.set(false);
        query.set(String::new());
        if let Some(el) = input_ref.get() { let _ = el.focus(); }
    };
    let replay_demo = move |_| {
        demo_state.set(DemoState {
            query_idx: 0, phase: DemoPhase::Typing, char_pos: 0, ticks_left: 0,
        });
        query.set(String::new());
        demo_running.set(true);
    };

    let has_text = move || !query.get().is_empty();
    let shimmer_class = move || if searching.get() { "sb-shimmer visible" } else { "sb-shimmer" };

    let non_leaving_count = move || results.with(|v| v.iter().filter(|r| !r.leaving.get()).count());
    let results_label = move || {
        let q = committed_query.get();
        let n = non_leaving_count();
        if q.trim().is_empty() { "Recent".to_string() }
        else if n == 0 { "0 results".to_string() }
        else if n == 1 { "1 result".to_string() }
        else { format!("{} results", n) }
    };
    let show_no_matches = move || {
        if committed_query.get().trim().is_empty() { return false; }
        results.with(|v| v.iter().all(|r| r.leaving.get()))
    };

    view! {
        <div class="sb-root">
            <div class="sb-toolbar">
                <button class="btn" on:click=replay_demo>"Replay demo"</button>
            </div>
            <div class="sb-page mock-page">
                <div class="sb-input-row">
                    <svg class="sb-magnifier" viewBox="0 0 24 24" width="18" height="18" aria-hidden="true">
                        <circle cx="10" cy="10" r="6" />
                        <line x1="15" y1="15" x2="20" y2="20" />
                    </svg>
                    <input
                        node_ref=input_ref
                        class="sb-input"
                        type="text"
                        placeholder="Search pages and notes…"
                        autocomplete="off"
                        spellcheck="false"
                        aria-label="Search"
                        on:input=on_input
                        on:focus=on_focus
                    />
                    {move || if !has_text() {
                        view! { <kbd class="sb-kbd">"/"</kbd> }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }}
                    {move || if has_text() {
                        view! {
                            <button class="sb-clear" aria-label="Clear search" on:click=clear>
                                "\u{00d7}"
                            </button>
                        }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }}
                </div>
                <div class=shimmer_class></div>
                <div class="sb-results-card">
                    <div class="sb-results-head">
                        <span class="label">{results_label}</span>
                    </div>
                    <div class="sb-results-body" node_ref=body_ref>
                        <For
                            each=move || results.get()
                            key=|r| r.idx
                            children=move |row: DisplayRow| {
                                render_row(row, results, committed_query)
                            }
                        />
                        {move || if show_no_matches() {
                            view! {
                                <div class="sb-no-matches">
                                    "No pages match, try 'trip' or 'budget'."
                                </div>
                            }.into_any()
                        } else {
                            view! { <span></span> }.into_any()
                        }}
                    </div>
                </div>
                <div class="sb-footer-hint">
                    "Type to search, or press the replay button to see the demo again."
                </div>
            </div>
        </div>
    }
}

fn render_row(
    row: DisplayRow,
    results: RwSignal<Vec<DisplayRow>>,
    committed_query: RwSignal<String>,
) -> impl IntoView {
    let idx = row.idx;
    let doc = &DOCS[idx];
    let title = doc.title;
    let breadcrumb = doc.breadcrumb;
    let snippet = doc.snippet;
    let leaving = row.leaving;
    let row_style = format!("--i: {}; --base-delay: {}ms;", row.enter_i, row.enter_delay_ms);

    let row_class = move || if leaving.get() { "sb-row leaving" } else { "sb-row" };
    let result_class = move || {
        let first_non_leaving = results.with(|v|
            v.iter().find(|r| !r.leaving.get()).map(|r| r.idx)
        );
        if first_non_leaving == Some(idx) { "sb-result sb-result-active" } else { "sb-result" }
    };

    view! {
        <div class=row_class style=row_style.clone()>
            <div class="sb-row-clip">
                <div class=result_class style=row_style.clone()>
                    <div class="sb-result-title">{title}</div>
                    <div class="sb-result-breadcrumb">{breadcrumb}</div>
                    <div class="sb-result-snippet">
                        {move || {
                            let q = committed_query.get();
                            let segments = highlight_segments(snippet, &q);
                            segments.into_iter().map(|(text, is_match)| {
                                if is_match {
                                    view! { <span class="sb-mark">{text}</span> }.into_any()
                                } else {
                                    view! { <span>{text}</span> }.into_any()
                                }
                            }).collect_view()
                        }}
                    </div>
                </div>
            </div>
        </div>
    }
}