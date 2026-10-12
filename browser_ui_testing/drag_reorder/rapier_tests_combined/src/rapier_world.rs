use rapier2d::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Page hit regions are stored in the world with ids at or above this value.
/// Block ids stay strictly below it.
pub const PAGE_ID_BASE: usize = 1000;

/// Every block lives as a fixed rapier body. The world is never stepped —
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

    /// Replace the world's contents with the given boxes. Blocks use ids below
    /// PAGE_ID_BASE; page regions use PAGE_ID_BASE + index.
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

    /// Closest block above and below `rect` by center-Y, requiring horizontal
    /// overlap with `rect`'s width inflated by `band` on each side. Ids in
    /// `exclude_ids` and page regions are ignored. Kept for parity with
    /// rapier_test_2; nothing in the combined crate calls it.
    #[allow(dead_code)]
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
            if id >= PAGE_ID_BASE {
                continue;
            }
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

    /// Ids of blocks whose AABB center x lies inside the given page's region,
    /// sorted by center y ascending. Page regions themselves are excluded.
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
            let cy = (aabb.mins.y + aabb.maxs.y) / 2.0;
            if cx >= rx0 && cx <= rx1 && cy >= ry0 && cy <= ry1 {
                hits.push((cy, id));
            }
        }
        hits.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        hits.into_iter().map(|(_, id)| id).collect()
    }

    /// Which page a point falls into, and the id of the block in that page
    /// (excluding `exclude_ids`) whose center y is the greatest one still
    /// above the point, or None if no block is above. Region edges are
    /// inclusive. Returns None when the point is in no page region.
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
            // Page hit is horizontal only. Vertical position picks the slot
            // inside the page, never whether the page counts.
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
