//! Caller pov: non-reactive state for one drag in progress, plus the
//! DOM helpers that read row rects and scroll position out of the
//! browser. Not a signal — the fields change on every pointer move and
//! nothing should re-render because of them. The reactive view is
//! shared_owner::drag_in_progress_feedback.
//!
//! Content coordinates are what the rest of the drag math uses: (top,
//! bottom) measured relative to the scroll body's top plus the current
//! scrollTop. That way a rect stays put in content coordinates even if
//! the body scrolls, so the insertion-slot math sees a stable set of
//! midpoints.
//!
//! Every DOM call is wrapped so errors carry err_msg, method and file.
//! The alternative — letting web-sys errors surface as bare JsValues or
//! panics — would be much harder to debug when a selector stops
//! matching.

use std::fmt;

use web_sys::{wasm_bindgen::JsCast, Element, HtmlElement};

#[derive(Clone, Copy, Debug)]
pub struct RowRectInContentCoords {
    pub row_id: usize,
    pub top: f32,
    pub bottom: f32,
}

impl RowRectInContentCoords {
    pub fn midpoint(&self) -> f32 {
        (self.top + self.bottom) / 2.0
    }
}

#[derive(Clone)]
pub struct DragSession {
    pub dragged_row_id: usize,
    pub start_pointer_y: f32,
    pub start_scroll_top: f32,
    pub last_pointer_y: f32,
    pub moved: bool,
    pub row_rects: Vec<RowRectInContentCoords>,
}

#[derive(Debug)]
pub struct DomReadError {
    pub err_msg: String,
    pub method: &'static str,
    pub file: &'static str,
}

impl fmt::Display for DomReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}: {}", self.method, self.file, self.err_msg)
    }
}

pub fn read_row_rects_in_content_coords(
    scroll_body: &HtmlElement,
) -> Result<Vec<RowRectInContentCoords>, DomReadError> {
    const M: &str = "read_row_rects_in_content_coords";
    // Attribute selector is deliberate. The header and footer cells
    // carry the same dv-cell-right class as the row cells but have no
    // data-row-id, so the bare class selector would match them and the
    // parse step would fail on the very first node.
    let nodes = scroll_body
        .query_selector_all(".dv-cell.dv-cell-right[data-row-id]")
        .map_err(|_| DomReadError {
            err_msg: "query_selector_all failed for .dv-cell.dv-cell-right[data-row-id]"
                .into(),
            method: M,
            file: file!(),
        })?;
    let body_rect = scroll_body.get_bounding_client_rect();
    let body_top = body_rect.top() as f32;
    let scroll_top = scroll_body.scroll_top() as f32;
    let mut out = Vec::with_capacity(nodes.length() as usize);
    for i in 0..nodes.length() {
        let node = nodes.get(i).ok_or_else(|| DomReadError {
            err_msg: format!("node at index {i} is missing"),
            method: M,
            file: file!(),
        })?;
        let el: &Element = node.dyn_ref::<Element>().ok_or_else(|| DomReadError {
            err_msg: format!("node at index {i} is not an Element"),
            method: M,
            file: file!(),
        })?;
        let row_id_attr = el.get_attribute("data-row-id").ok_or_else(|| DomReadError {
            err_msg: format!("right cell at index {i} has no data-row-id"),
            method: M,
            file: file!(),
        })?;
        let row_id: usize = row_id_attr.parse().map_err(|_| DomReadError {
            err_msg: format!("data-row-id '{row_id_attr}' is not a number"),
            method: M,
            file: file!(),
        })?;
        let r = el.get_bounding_client_rect();
        let top = (r.top() as f32) - body_top + scroll_top;
        let bottom = (r.bottom() as f32) - body_top + scroll_top;
        out.push(RowRectInContentCoords { row_id, top, bottom });
    }
    Ok(out)
}

pub fn read_scroll_top(scroll_body: &HtmlElement) -> f32 {
    scroll_body.scroll_top() as f32
}

pub fn read_pointer_y(e: &leptos::ev::MouseEvent) -> f32 {
    e.client_y() as f32
}

pub fn auto_scroll_if_pointer_is_near_the_edge(
    scroll_body: &HtmlElement,
    pointer_client_y: f32,
) {
    const EDGE_ZONE_PX: f32 = 48.0;
    const STEP_PX: f32 = 8.0;
    let rect = scroll_body.get_bounding_client_rect();
    let top = rect.top() as f32;
    let bottom = rect.bottom() as f32;
    if pointer_client_y - top < EDGE_ZONE_PX {
        scroll_body.scroll_by_with_x_and_y(0.0, -(STEP_PX as f64));
    } else if bottom - pointer_client_y < EDGE_ZONE_PX {
        scroll_body.scroll_by_with_x_and_y(0.0, STEP_PX as f64);
    }
}
