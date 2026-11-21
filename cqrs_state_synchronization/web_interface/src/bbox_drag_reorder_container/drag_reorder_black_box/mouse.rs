use leptos::ev;
use leptos::html::Div;
use leptos::prelude::*;

use super::apply_move::apply_move;
use super::drag_state::{DRAG_THRESHOLD, DragState, RectDragState};
use super::layout::{BLOCK_W, Block, RECT_H, RECT_W, VACUUM_PROXIMITY, VACUUM_W};
use super::rapier_world::RapierWorld;
use super::types::MoveEvent;

pub fn handle_mouse_move(
    e: ev::MouseEvent,
    stage_ref: NodeRef<Div>,
    drag: RwSignal<Option<DragState>>,
    rect_drag: RwSignal<Option<RectDragState>>,
) {
    let Some(stage) = stage_ref.get() else {
        return;
    };
    let rect = stage.get_bounding_client_rect();
    let mx = e.client_x() as f32 - rect.left() as f32;
    let my = e.client_y() as f32 - rect.top() as f32;

    if let Some(mut d) = drag.get() {
        if !d.moved
            && ((mx - d.start.0).abs() > DRAG_THRESHOLD || (my - d.start.1).abs() > DRAG_THRESHOLD)
        {
            d.moved = true;
        }
        d.current = (mx - d.offset.0, my - d.offset.1);
        drag.set(Some(d));
    }

    if let Some(mut r) = rect_drag.get() {
        if !r.moved
            && ((mx - r.start.0).abs() > DRAG_THRESHOLD || (my - r.start.1).abs() > DRAG_THRESHOLD)
        {
            r.moved = true;
        }
        r.current = (mx - r.offset.0, my - r.offset.1);
        rect_drag.set(Some(r));
    }
}

#[allow(clippy::too_many_arguments)]
pub fn handle_mouse_up(
    e: ev::MouseEvent,
    stage_ref: NodeRef<Div>,
    drag: RwSignal<Option<DragState>>,
    rect_drag: RwSignal<Option<RectDragState>>,
    boxes: RwSignal<Vec<Block>>,
    stored: RwSignal<Option<usize>>,
    page_count: RwSignal<usize>,
    current_page_ids: RwSignal<Vec<String>>,
    rapier: &RapierWorld,
    on_move: Callback<MoveEvent>,
) {
    let Some(stage) = stage_ref.get() else {
        drag.set(None);
        rect_drag.set(None);
        return;
    };
    let rect = stage.get_bounding_client_rect();
    let ox = rect.left() as f32;
    //let oy = rect.top() as f32;
    let mx = e.client_x() as f32 - ox;
    //let my = e.client_y() as f32 - oy;

    if let Some(d) = drag.get_untracked() {
        let snapshot = boxes.get_untracked();
        let dragged = snapshot.iter().find(|b| b.internal_id == d.id).cloned();
        if let Some(dragged) = dragged {
            let in_vacuum = mx <= VACUUM_W + VACUUM_PROXIMITY;

            if !d.moved {
                // click with no movement: do nothing
            } else if in_vacuum {
                let stored_id = stored.get_untracked();
                if stored_id != Some(d.id) {
                    let mut new_boxes = boxes.get_untracked();
                    if let Some(prev) = stored_id {
                        if let Some(pb) = new_boxes.iter_mut().find(|b| b.internal_id == prev) {
                            pb.ghost = false;
                        }
                    }
                    if let Some(db) = new_boxes.iter_mut().find(|b| b.internal_id == d.id) {
                        db.ghost = true;
                    }
                    boxes.set(new_boxes);
                    stored.set(Some(d.id));
                }
            } else if let Some((page, above)) = rapier.locate_drop(
                (d.current.0 + BLOCK_W / 2.0, d.current.1 + dragged.h / 2.0),
                &[d.id],
            ) {
                apply_move(
                    d.id,
                    page,
                    above,
                    false,
                    boxes,
                    rapier,
                    stored,
                    page_count,
                    current_page_ids,
                    on_move,
                );
            }
        }
        drag.set(None);
    }

    if let Some(rd) = rect_drag.get_untracked() {
        if rd.moved {
            if let Some(stored_id) = stored.get_untracked() {
                if let Some((page, above)) = rapier.locate_drop(
                    (rd.current.0 + RECT_W / 2.0, rd.current.1 + RECT_H / 2.0),
                    &[stored_id],
                ) {
                    apply_move(
                        stored_id,
                        page,
                        above,
                        true,
                        boxes,
                        rapier,
                        stored,
                        page_count,
                        current_page_ids,
                        on_move,
                    );
                }
            }
        }
        rect_drag.set(None);
    }
}
