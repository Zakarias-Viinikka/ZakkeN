use rapier2d::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub const PAGE_ID_BASE: usize = 1_000_000_000;

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

    pub fn page_order(&self, page: usize) -> Vec<usize> {
        let inner = self.inner.borrow();
        let region_id = PAGE_ID_BASE + page;

        let mut region: Option<(f32, f32, f32, f32)> = None;
        for (ch, collider) in inner.colliders.iter() {
            if inner.collider_to_id.get(&ch) == Some(&region_id) {
                let aabb = collider.compute_aabb();
                region = Some((aabb.mins.x, aabb.maxs.x, aabb.mins.y, aabb.maxs.y));
                break;
            }
        }
        let Some((rx0, rx1, ry0, ry1)) = region else {
            return Vec::new();
        };

        let mut hits: Vec<(f32, usize)> = Vec::new();
        for (ch, collider) in inner.colliders.iter() {
            let Some(&id) = inner.collider_to_id.get(&ch) else {
                continue;
            };
            if id >= PAGE_ID_BASE {
                continue;
            }
            let aabb = collider.compute_aabb();
            let cx = (aabb.mins.x + aabb.maxs.x) / 2.0;
            let cy = aabb.mins.y;
            if cx >= rx0 && cx <= rx1 && cy >= ry0 && cy <= ry1 {
                hits.push((cy, id));
            }
        }
        hits.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        hits.into_iter().map(|(_, id)| id).collect()
    }

    pub fn locate_drop(
        &self,
        point: (f32, f32),
        exclude_ids: &[usize],
    ) -> Option<(usize, Option<usize>)> {
        let inner = self.inner.borrow();

        let mut found_page: Option<usize> = None;
        for (ch, collider) in inner.colliders.iter() {
            let Some(&id) = inner.collider_to_id.get(&ch) else {
                continue;
            };
            if id < PAGE_ID_BASE {
                continue;
            }
            let aabb = collider.compute_aabb();
            if point.0 >= aabb.mins.x && point.0 <= aabb.maxs.x {
                found_page = Some(id - PAGE_ID_BASE);
                break;
            }
        }
        let page = found_page?;

        let region_id = PAGE_ID_BASE + page;
        let mut region: Option<(f32, f32, f32, f32)> = None;
        for (ch, collider) in inner.colliders.iter() {
            if inner.collider_to_id.get(&ch) == Some(&region_id) {
                let aabb = collider.compute_aabb();
                region = Some((aabb.mins.x, aabb.maxs.x, aabb.mins.y, aabb.maxs.y));
                break;
            }
        }
        let Some((rx0, rx1, ry0, ry1)) = region else {
            return Some((page, None));
        };

        let mut best: Option<(f32, usize)> = None;
        for (ch, collider) in inner.colliders.iter() {
            let Some(&id) = inner.collider_to_id.get(&ch) else {
                continue;
            };
            if id >= PAGE_ID_BASE {
                continue;
            }
            if exclude_ids.contains(&id) {
                continue;
            }
            let aabb = collider.compute_aabb();
            let cx = (aabb.mins.x + aabb.maxs.x) / 2.0;
            let cy = (aabb.mins.y + aabb.maxs.y) / 2.0;
            if cx < rx0 || cx > rx1 || cy < ry0 || cy > ry1 {
                continue;
            }
            if cy < point.1 {
                if best.map_or(true, |(d, _)| cy > d) {
                    best = Some((cy, id));
                }
            }
        }

        Some((page, best.map(|(_, id)| id)))
    }
}
