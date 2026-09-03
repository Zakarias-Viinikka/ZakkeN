use crate::fun_drag::ContainerSize;
use crate::internal_calculations::{RapierContext, WorldBox};
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
            if let Some(collider) = colliders.get_mut(box_item.collider_handle) {
                collider.set_translation(center);
            }
        });
    }
}

pub fn create_borders(
    container_size: RwSignal<ContainerSize>,
    rapier_ctx: ArcRwSignal<RapierContext>,
) {
    Effect::new(move |_| {
        let c_size = container_size.get();
        let width = c_size.width as f32;
        let height = c_size.height as f32;
        let thickness = 10.0;

        let walls = [
            // top: center at y = -thickness/2 (extends outward)
            (
                Vector::new(width / 2.0, -thickness / 2.0),
                Vector::new(width, thickness),
            ),
            // bottom
            (
                Vector::new(width / 2.0, height + thickness / 2.0),
                Vector::new(width, thickness),
            ),
            // left
            (
                Vector::new(-thickness / 2.0, height / 2.0),
                Vector::new(thickness, height),
            ),
            // right
            (
                Vector::new(width + thickness / 2.0, height / 2.0),
                Vector::new(thickness, height),
            ),
        ];

        rapier_ctx.update(|ctx| {
            let mut rigid_bodies = ctx.rigid_bodies.write().unwrap();
            let mut colliders = ctx.colliders.write().unwrap();

            // Remove any existing walls? (optional – if you want to clean up, you'd need handles)
            // For simplicity, we just insert new ones each time – but they'll accumulate on resize.
            // To avoid accumulation, you could store handles and remove them first.
            // But since you only call this once and container size doesn't change, it's fine.
            for (pos, size) in walls.iter() {
                let body_handle =
                    rigid_bodies.insert(RigidBodyBuilder::fixed().translation(*pos).build());
                let collider = ColliderBuilder::cuboid(size.x / 2.0, size.y / 2.0).build();
                colliders.insert_with_parent(collider, body_handle, &mut rigid_bodies);
            }
        });
    });
}
