use rapier2d::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Rapier2d wrapper used purely as a spatial query engine. All bodies are
/// fixed. The world is never stepped; motion is driven by the DOM and
/// rapier only answers "what is nearest in this direction".
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

    pub fn rebuild(&self, boxes: &[(usize, (f32, f32), (f32, f32))]) {
        let mut inner = self.inner.borrow_mut();
        inner.rigid_bodies = RigidBodySet::new();
        inner.colliders = ColliderSet::new();
        inner.id_to_body.clear();
        inner.collider_to_id.clear();

        for &(id, (x, y), (w, h)) in boxes {
            let body = RigidBodyBuilder::fixed()
                .translation(Vec2::new(x + w / 2.0, y + h / 2.0))
                .build();
            let body_handle = inner.rigid_bodies.insert(body);
            let collider = ColliderBuilder::cuboid(w / 2.0, h / 2.0).build();
            let Inner {
                rigid_bodies,
                colliders,
                id_to_body,
                collider_to_id,
            } = &mut *inner;
            let ch = colliders.insert_with_parent(collider, body_handle, rigid_bodies);
            id_to_body.insert(id, body_handle);
            collider_to_id.insert(ch, id);
        }
    }

    /// Nearest box in direction `dir` (-1 up, +1 down) from `rect`, ignoring
    /// ids in `exclude_ids`. Returns `(id, gap)` where `gap` is the distance
    /// from the moving rect's leading edge to the neighbour's near edge.
    /// Negative gap means overlap. Only boxes that horizontally overlap
    /// `rect` are considered.
    pub fn nearest_in_direction(
        &self,
        rect: ((f32, f32), (f32, f32)),
        dir: i8,
        exclude_ids: &[usize],
    ) -> Option<(usize, f32)> {
        if dir == 0 {
            return None;
        }
        let inner = self.inner.borrow();
        let (left, top, w, h) = (rect.0 .0, rect.0 .1, rect.1 .0, rect.1 .1);
        let right = left + w;
        let bottom = top + h;
        let center_y = top + h / 2.0;

        let mut best: Option<(f32, usize, f32)> = None;

        for (ch, collider) in inner.colliders.iter() {
            let Some(&id) = inner.collider_to_id.get(&ch) else {
                continue;
            };
            if exclude_ids.contains(&id) {
                continue;
            }

            let aabb = collider.compute_aabb();
            let (b_min_x, b_max_x) = (aabb.mins.x, aabb.maxs.x);
            let (b_min_y, b_max_y) = (aabb.mins.y, aabb.maxs.y);

            if b_max_x <= left || b_min_x >= right {
                continue;
            }

            let other_center_y = (b_min_y + b_max_y) / 2.0;

            let gap = if dir < 0 {
                if other_center_y >= center_y {
                    continue;
                }
                top - b_max_y
            } else {
                if other_center_y <= center_y {
                    continue;
                }
                b_min_y - bottom
            };

            let dist = (other_center_y - center_y).abs();
            if best.map_or(true, |(d, _, _)| dist < d) {
                best = Some((dist, id, gap));
            }
        }

        best.map(|(_, id, gap)| (id, gap))
    }
}
