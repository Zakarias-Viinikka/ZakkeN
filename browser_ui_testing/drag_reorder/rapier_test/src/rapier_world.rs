use rapier2d::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Every box lives as a fixed rapier body. The world is never stepped —
/// the DOM handles motion, rapier answers spatial queries.
#[derive(Clone)]
pub struct RapierWorld {
    inner: Rc<RefCell<Inner>>,
}

struct Inner {
    rigid_bodies: RigidBodySet,
    colliders: ColliderSet,
    id_to_body: HashMap<usize, RigidBodyHandle>,
    collider_to_id: HashMap<ColliderHandle, usize>,
}

impl RapierWorld {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(Inner {
                rigid_bodies: RigidBodySet::new(),
                colliders: ColliderSet::new(),
                id_to_body: HashMap::new(),
                collider_to_id: HashMap::new(),
            })),
        }
    }

    pub fn add_fixed_box(&self, id: usize, home: (f32, f32), size: (f32, f32)) {
        let mut inner = self.inner.borrow_mut();
        let body = RigidBodyBuilder::fixed()
            .translation(Vec2::new(home.0 + size.0 / 2.0, home.1 + size.1 / 2.0))
            .build();
        let body_handle = inner.rigid_bodies.insert(body);
        let collider = ColliderBuilder::cuboid(size.0 / 2.0, size.1 / 2.0).build();
        let Inner {
            rigid_bodies,
            colliders,
            id_to_body,
            collider_to_id,
        } = &mut *inner;
        let collider_handle = colliders.insert_with_parent(collider, body_handle, rigid_bodies);
        id_to_body.insert(id, body_handle);
        collider_to_id.insert(collider_handle, id);
    }

    /// Ask rapier for the closest box above and the closest box below the
    /// given rect (top-left, size), requiring horizontal overlap. The box
    /// with `exclude_id` is never returned.
    pub fn find_above_and_below(
        &self,
        rect: ((f32, f32), (f32, f32)),
        exclude_ids: &[usize],
    ) -> (Option<usize>, Option<usize>) {
        let inner = self.inner.borrow();
        let (left, top, w, h) = (rect.0 .0, rect.0 .1, rect.1 .0, rect.1 .1);
        let right = left + w;
        let center_y = top + h / 2.0;

        let mut above: Option<(f32, usize)> = None;
        let mut below: Option<(f32, usize)> = None;

        for (collider_handle, collider) in inner.colliders.iter() {
            let Some(&id) = inner.collider_to_id.get(&collider_handle) else {
                continue;
            };
            if exclude_ids.contains(&id) {
                continue;
            }

            let aabb = collider.compute_aabb();
            let b_min = aabb.mins;
            let b_max = aabb.maxs;

            // Horizontal overlap required.
            if b_max.x <= left || b_min.x >= right {
                continue;
            }

            // Compare centers so vertically overlapping boxes still resolve.
            let other_center_y = (b_min.y + b_max.y) / 2.0;
            let dist = (other_center_y - center_y).abs();

            if other_center_y < center_y {
                if above.map_or(true, |(d, _)| dist < d) {
                    above = Some((dist, id));
                }
            } else if other_center_y > center_y {
                if below.map_or(true, |(d, _)| dist < d) {
                    below = Some((dist, id));
                }
            }
        }

        (above.map(|(_, id)| id), below.map(|(_, id)| id))
    }
}