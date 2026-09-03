#![allow(unused_labels)]
use leptos::logging::log;
use leptos::prelude::*;
use rapier2d::prelude::*;

use crate::fun_drag::{BoxSettings, ContainerSize, ImmovableObjectSettings};

#[derive(Clone, Debug)]
pub struct WorldBox {
    pub id: u32,
    pub width: u32,
    pub height: u32,
    pub position: RwSignal<(f32, f32)>,
    pub animation_state: RwSignal<AnimationState>,
    pub velocity: RwSignal<Vec2>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AnimationState {
    ActivelyDragged,
    TouchingMouse,
    PassivelyMoving,
    Still,
}

#[derive(Clone, Debug)]
pub struct ActivelyMovingBoxes {
    pub box_ids: Vec<u32>,
}

// Collision detection only – no physics stepping.
pub fn check_if_colliding_with_another_box(
    colliders: &ColliderSet,
    position: (u32, u32),
    box_settings: &BoxSettings,
) -> bool {
    let new_left = position.0 as f32;
    let new_top = position.1 as f32;
    let new_right = new_left + box_settings.width as f32;
    let new_bottom = new_top + box_settings.height as f32;

    for (_, collider) in colliders.iter() {
        let Some(cuboid) = collider.shape().as_cuboid() else {
            continue;
        };
        let center = collider.position().translation;
        let existing_left = center.x - cuboid.half_extents.x;
        let existing_top = center.y - cuboid.half_extents.y;
        let existing_right = center.x + cuboid.half_extents.x;
        let existing_bottom = center.y + cuboid.half_extents.y;

        let overlapping = new_left < existing_right
            && new_right > existing_left
            && new_top < existing_bottom
            && new_bottom > existing_top;

        if overlapping {
            return true;
        }
    }
    false
}

/*
//DBG
use std::sync::atomic::{AtomicU64, Ordering};

static DBG_CTR: AtomicU64 = AtomicU64::new(0);
//DBG
*/

pub fn update_world(
    colliders: RwSignal<ColliderSet>,
    container_size: RwSignal<ContainerSize>,
    mouse_pos: ReadSignal<(f32, f32)>,
    boxes: ReadSignal<Vec<WorldBox>>,
    set_boxes: WriteSignal<Vec<WorldBox>>,
    actively_moving_boxes: ReadSignal<ActivelyMovingBoxes>,
    actively_moving_boxes_set: WriteSignal<ActivelyMovingBoxes>,
    immovable_obj_settings: RwSignal<ImmovableObjectSettings>,
) {
    /*
    // Increment debug counter
    let call_count = DBG_CTR.fetch_add(1, Ordering::Relaxed) + 1;

    // Log every 60 calls (roughly once per second at 60 FPS)
    if call_count % 60 == 0 {
        let boxes_snapshot = boxes.get_untracked();
        let active_ids = &actively_moving_boxes.get_untracked().box_ids;

        if let Some(first_box) = boxes_snapshot.iter().find(|b| active_ids.contains(&b.id)) {
            log!(
                "DBG #{}, first active box: id={}, pos={:?}, vel={:?}, state={:?}",
                call_count,
                first_box.id,
                first_box.position.get_untracked(),
                first_box.velocity.get_untracked(),
                first_box.animation_state.get_untracked(),
            );
        } else {
            log!("DBG #{}, no active boxes", call_count);
        }
    }
    */

    let mouse = mouse_pos.get_untracked();
    // Original update loop
    for box_id in actively_moving_boxes.get_untracked().box_ids.iter() {
        'block: {
            let boxes = boxes.get_untracked();
            let box_item = if let Some(item) = boxes.iter().find(|box_item| &box_item.id == box_id)
            {
                item
            } else {
                break 'block;
            };

            let is_touching = || {
                let current_pos = box_item.position.get_untracked();
                is_box_touching_mouse(IsBoxTouchingMouseCtx {
                    mouse_x: mouse.0,
                    mouse_y: mouse.1,
                    box_x: current_pos.0,
                    box_y: current_pos.1,
                    box_height: box_item.height as f32,
                    box_width: box_item.width as f32,
                })
            };

            let mouse = mouse_pos.get_untracked();
            let pos = box_item.position.get_untracked();

            let figure_out_drag_ctx = FigureOutDrag {
                box_x_middle: pos.0 + (box_item.width as f32) / 2.0,
                box_y_middle: pos.1 + (box_item.height as f32) / 2.0,
                mouse_x: mouse.0,
                mouse_y: mouse.1,
                old_velocity: box_item.velocity.get_untracked(),
            };

            match box_item.animation_state.get_untracked() {
                AnimationState::ActivelyDragged => {
                    let new_velocity = figure_out_new_drag_velocity(figure_out_drag_ctx);
                    box_item.velocity.set(new_velocity);
                }
                AnimationState::Still => {
                    break 'block;
                }
                AnimationState::TouchingMouse => {
                    if !is_touching() {
                        box_item
                            .animation_state
                            .set(AnimationState::ActivelyDragged);
                    } else {
                        crawl_to_a_stop(figure_out_drag_ctx, box_item.id, set_boxes);
                    }
                }
                _ => break 'block, // todo
            }

            // --- move the box --- //
            let current_pos = box_item.position.get_untracked();
            let velocity = box_item.velocity.get_untracked();
            let new_pos = (current_pos.0 + velocity.x, current_pos.1 + velocity.y);
            box_item.position.set(new_pos);
            // Optional: apply damping (e.g., reduce velocity)
            // box_item.velocity.set(velocity * 0.9);
            // --- move the box --- //

            // -- check if the box is touching the cursor after movement is applied //
            if AnimationState::ActivelyDragged == box_item.animation_state.get_untracked() {
                if is_touching() {
                    box_item.animation_state.set(AnimationState::TouchingMouse);
                }
            }
        }
    }
}

fn crawl_to_a_stop(ctx: FigureOutDrag, box_id: u32, set_boxes: WriteSignal<Vec<WorldBox>>) {
    let mut ctx = ctx;
    ctx.old_velocity = ctx.old_velocity * 0.7;

    set_boxes.update(|boxes_vec| {
        if let Some(box_item) = boxes_vec.iter_mut().find(|b| b.id == box_id) {
            let dx = ctx.box_x_middle - ctx.mouse_x;
            let dy = ctx.box_y_middle - ctx.mouse_y;
            let distance = (dx * dx + dy * dy).sqrt();

            // Stop completely when very close to the mouse center
            const DEAD_ZONE: f32 = 3.0; // adjust pixels as needed
            if distance < DEAD_ZONE {
                box_item.velocity.set(Vec2::new(0.0, 0.0));
                return;
            }

            // Otherwise, apply dynamic minimum speed
            let min_speed = distance * 0.2;
            let new_vel = figure_out_new_drag_velocity(ctx);
            let speed = new_vel.length();

            if speed < min_speed {
                let dir = new_vel.try_normalize().unwrap_or(Vec2::new(0.0, 0.0));
                box_item.velocity.set(dir * min_speed);
            } else {
                box_item.velocity.set(new_vel);
            }
        }
    });
}

struct IsBoxTouchingMouseCtx {
    mouse_x: f32,
    mouse_y: f32,
    box_x: f32,
    box_y: f32,
    box_height: f32,
    box_width: f32,
}

fn is_box_touching_mouse(ctx: IsBoxTouchingMouseCtx) -> bool {
    let within_x = ctx.mouse_x >= ctx.box_x && ctx.mouse_x <= ctx.box_x + ctx.box_width;
    let within_y = ctx.mouse_y >= ctx.box_y && ctx.mouse_y <= ctx.box_y + ctx.box_height;
    within_x && within_y
}

struct FigureOutDrag {
    box_x_middle: f32,
    box_y_middle: f32,
    mouse_x: f32,
    mouse_y: f32,
    old_velocity: Vec2,
}

fn figure_out_new_drag_velocity(ctx: FigureOutDrag) -> Vec2 {
    let new_velocity = {
        let old = ctx.old_velocity;
        let desired = desired_drag_velocity(
            (ctx.mouse_x, ctx.mouse_y),
            (ctx.box_x_middle, ctx.box_y_middle),
        );

        let new_x = (old.x + desired.x) / 2.0;
        let new_y = (old.y + desired.y) / 2.0;
        Vec2::new(new_x, new_y)
    };
    new_velocity
}

fn desired_drag_velocity(mouse_pos: (f32, f32), item_pos: (f32, f32)) -> Vec2 {
    let dx = mouse_pos.0 - item_pos.0;
    let dy = mouse_pos.1 - item_pos.1;

    let max_speed = 10.0;
    let speed_factor = 5.0;

    let mut desired = Vec2::new(dx * speed_factor, dy * speed_factor);
    if desired.length() > max_speed {
        desired = desired.normalize() * max_speed;
    }
    desired
}
