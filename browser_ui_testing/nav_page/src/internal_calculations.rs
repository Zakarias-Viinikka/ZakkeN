use leptos::prelude::*;
use rapier2d::prelude::*;

use crate::fun_drag::{BoxSettings, ContainerSize};

#[derive(Clone)]
pub struct WorldBox {
    pub id: u32,
    pub width: u32,
    pub height: u32,
    pub position: RwSignal<(f32, f32)>,
    pub animation_state: AnimationState,
}

#[derive(Clone)]
pub enum AnimationState {
    ActivelyDragged,
    TouchingMouse,
    PassivelyMoving,
    Still,
}

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

// Placeholder for your custom update logic.
// You will implement movement, velocity, and collision response here.
pub fn update_world(
    colliders: RwSignal<ColliderSet>,
    container_size: RwSignal<ContainerSize>,
    mouse_pos: ReadSignal<(f32, f32)>,
    boxes: ReadSignal<Vec<WorldBox>>,
    dt: f32,
    actively_moving_boxes: ReadSignal<ActivelyMovingBoxes>,
    actively_moving_boxes_set: WriteSignal<ActivelyMovingBoxes>,
) {
    //in the middle of starting to write this out.
    //for box_to_update in actively_moving_boxes.get().iter() {}
}
