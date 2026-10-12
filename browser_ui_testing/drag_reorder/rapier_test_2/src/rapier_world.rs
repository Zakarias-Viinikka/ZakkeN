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

    /// Replace the world's contents with the given boxes.
    pub fn rebuild(&self, boxes: &[(usize, (f32, f32), (f32, f32))]) {
        let mut inner = self.inner.borrow_mut();
        inner.rigid_bodies = RigidBodySet::new();
        inner.colliders = ColliderSet::new();
        inner.id_to_body.clear();
        inner.collider_to_id.clear();

        for &(id, home, size) in boxes {
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
    }

    /// Closest box above and below `rect` by center-Y, requiring horizontal
    /// overlap with `rect`'s width inflated by `band` on each side. Ids in
    /// `exclude_ids` are ignored.
    pub fn find_above_and_below(
        &self,
        rect: ((f32, f32), (f32, f32)),
        exclude_ids: &[usize],
        band: f32,
    ) -> (Option<usize>, Option<usize>) {
        let inner = self.inner.borrow();
        let (left, top, w, h) = (rect.0 .0, rect.0 .1, rect.1 .0, rect.1 .1);
        let left_band = left - band;
        let right_band = left + w + band;
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

            if b_max.x <= left_band || b_min.x >= right_band {
                continue;
            }

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