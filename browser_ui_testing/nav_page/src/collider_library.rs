use crate::internal_calculations::WorldBox;
use leptos::prelude::*;
use rapier2d::math::Vec2;
use rapier2d::prelude::*;

pub fn update_collider_set(
    colliders: RwSignal<ColliderSet>,
    boxes: &Vec<WorldBox>,
    box_id: u32,
    new_pos: Vec2,
) {
    // Find the box dimensions
    let box_item = boxes.iter().find(|b| b.id == box_id);
    if let Some(box_item) = box_item {
        let center = Vec2::new(
            new_pos.x + box_item.width as f32 / 2.0,
            new_pos.y + box_item.height as f32 / 2.0,
        );
        colliders.update(|colliders| {
            let handle = ColliderHandle::from_raw_parts(box_id, 0);
            if let Some(collider) = colliders.get_mut(handle) {
                collider.set_translation(center);
            }
        });
    }
}

/*
pub fn do_i_collide_with_anyone_else(
    colliders: &ColliderSet,
    box_id: u32,
    boxes: &Vec<WorldBox>,
) -> bool {
    let my_handle = ColliderHandle::from_raw_parts(box_id, 0);
    let my_collider = match colliders.get(my_handle) {
        Some(c) => c,
        None => return false,
    };

    for other in boxes {
        if other.id == box_id {
            continue;
        }
        let other_handle = ColliderHandle::from_raw_parts(other.id, 0);
        if let Some(other_collider) = colliders.get(other_handle) {
            if my_collider.intersects(other_collider) {
                return true;
            }
        }
    }
    false
}

pub fn do_i_collide_with_the_immovable_object(
    colliders: &ColliderSet,
    box_id: u32,
    immovable_handle: ColliderHandle,
) -> bool {
    let my_handle = ColliderHandle::from_raw_parts(box_id, 0);
    let my_collider = match colliders.get(my_handle) {
        Some(c) => c,
        None => return false,
    };

    if let Some(imm_collider) = colliders.get(immovable_handle) {
        my_collider.intersects(imm_collider)
    } else {
        false
    }
} */
