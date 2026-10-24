use gloo_timers::future::TimeoutFuture;
use leptos::ev;
use leptos::html::Div;
use leptos::prelude::*;
use leptos::task::spawn_local;
use rapier2d::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::LazyLock;
use std::time::Duration;
use wasm_bindgen::{closure::Closure, JsCast};

static FOCUS_BLOCK_AT_START: LazyLock<ArcRwSignal<Option<String>>> =
    LazyLock::new(|| ArcRwSignal::new(None));

static FOCUS_BLOCK_ID: LazyLock<ArcRwSignal<Option<String>>> =
    LazyLock::new(|| ArcRwSignal::new(None));

const PAGE_ID_BASE: usize = 1_000_000_000;
const PAGE_W: f32 = 600.0;
const PAGE_TOP: f32 = 24.0;
const PAGE_START_X: f32 = 190.0;
const BLOCK_W: f32 = 520.0;
const BLOCK_PAD_X: f32 = 40.0;
const EDGE_PAD: f32 = 48.0;
const GAP: f32 = 16.0;
const STAGE_MIN_H: f32 = 720.0;
const BOTTOM_SLACK: f32 = 140.0;
const UNMEASURED_HEIGHT: f32 = 56.0;
const REGION_H: f32 = 20000.0;
const ACCEL: f32 = 1500.0;
const MAX_V: f32 = 600.0;
const DT: f32 = 0.016;
const CONTACT: f32 = 3.0;
const TRIGGER: f32 = GAP - CONTACT;
const DRAG_THRESHOLD: f32 = 4.0;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

#[derive(Clone, Debug, PartialEq)]
struct BlockData {
    id: String,
    content: String,
}

#[derive(Clone)]
struct BlockRenderInput {
    block_id: String,
    content: ArcRwSignal<String>,
    internal_id: usize,
    boxes: RwSignal<Vec<PhysicsBlock>>,
    rapier: StoredValue<RapierWorld, LocalStorage>,
}

struct BlockInputEvent {
    block_id: String,
    old_text: String,
    new_text: String,
    was_enter: bool,
    cursor_pos: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Phase {
    Idle,
    Waiting,
    Moving,
}

#[derive(Clone)]
struct PhysicsBlock {
    internal_id: usize,
    id: String,
    content: ArcRwSignal<String>,
    x: f32,
    y: f32,
    target_y: f32,
    vel: f32,
    h: f32,
    phase: Phase,
    prev: Option<usize>,
    measured: bool,
}

#[derive(Clone)]
struct DragState {
    id: usize,
    offset: (f32, f32),
    current: (f32, f32),
    moved: bool,
    start: (f32, f32),
}

#[derive(Clone)]
struct RapierWorld {
    inner: Rc<RefCell<RapierInner>>,
}

struct RapierInner {
    rigid_bodies: RigidBodySet,
    colliders: ColliderSet,
    id_to_body: HashMap<usize, RigidBodyHandle>,
    collider_to_id: HashMap<ColliderHandle, usize>,
}

impl RapierWorld {
    fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(RapierInner {
                rigid_bodies: RigidBodySet::new(),
                colliders: ColliderSet::new(),
                id_to_body: HashMap::new(),
                collider_to_id: HashMap::new(),
            })),
        }
    }

    fn rebuild(&self, entries: &[(usize, (f32, f32), (f32, f32))]) {
        let mut inner = self.inner.borrow_mut();
        inner.rigid_bodies = RigidBodySet::new();
        inner.colliders = ColliderSet::new();
        inner.id_to_body.clear();
        inner.collider_to_id.clear();

        for &(id, home, size) in entries {
            let body = RigidBodyBuilder::fixed()
                .translation(Vec2::new(
                    home.0 + size.0 / 2.0,
                    home.1 + size.1 / 2.0,
                ))
                .build();

            let body_handle = inner.rigid_bodies.insert(body);
            let collider =
                ColliderBuilder::cuboid(size.0 / 2.0, size.1 / 2.0).build();

            let RapierInner {
                rigid_bodies,
                colliders,
                id_to_body,
                collider_to_id,
            } = &mut *inner;

            let collider_handle =
                colliders.insert_with_parent(collider, body_handle, rigid_bodies);

            id_to_body.insert(id, body_handle);
            collider_to_id.insert(collider_handle, id);
        }
    }

    fn page_order(&self, page: usize) -> Vec<usize> {
        let inner = self.inner.borrow();
        let region_id = PAGE_ID_BASE + page;

        let mut region: Option<(f32, f32, f32, f32)> = None;

        for (ch, collider) in inner.colliders.iter() {
            if inner.collider_to_id.get(&ch) == Some(&region_id) {
                let aabb = collider.compute_aabb();
                region = Some((
                    aabb.mins.x,
                    aabb.maxs.x,
                    aabb.mins.y,
                    aabb.maxs.y,
                ));
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

        hits.sort_by(|a, b| a.0.total_cmp(&b.0));
        hits.into_iter().map(|(_, id)| id).collect()
    }

    fn locate_drop(
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
                region = Some((
                    aabb.mins.x,
                    aabb.maxs.x,
                    aabb.mins.y,
                    aabb.maxs.y,
                ));
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

            if id >= PAGE_ID_BASE || exclude_ids.contains(&id) {
                continue;
            }

            let aabb = collider.compute_aabb();
            let cx = (aabb.mins.x + aabb.maxs.x) / 2.0;
            let cy = (aabb.mins.y + aabb.maxs.y) / 2.0;

            if cx < rx0 || cx > rx1 || cy < ry0 || cy > ry1 {
                continue;
            }

            if cy < point.1 && best.map_or(true, |(distance, _)| cy > distance) {
                best = Some((cy, id));
            }
        }

        Some((page, best.map(|(_, id)| id)))
    }
}

fn page_x(page_index: usize) -> f32 {
    PAGE_START_X + page_index as f32 * (PAGE_W + 60.0)
}

fn block_x(page_index: usize) -> f32 {
    page_x(page_index) + BLOCK_PAD_X
}

fn stage_width() -> f32 {
    PAGE_START_X + PAGE_W + 40.0
}

fn page_panel_height(stage_h: f32) -> f32 {
    (stage_h - PAGE_TOP - 20.0).max(0.0)
}

fn current_height(block: &PhysicsBlock) -> f32 {
    if block.measured {
        block.h
    } else {
        UNMEASURED_HEIGHT
    }
}

fn stage_height(blocks: &[PhysicsBlock]) -> f32 {
    let mut height = STAGE_MIN_H;

    for block in blocks {
        let bottom = block.y.max(block.target_y) + current_height(block);
        let needed = bottom + BOTTOM_SLACK;
        if needed > height {
            height = needed;
        }
    }

    height
}

fn commit(
    blocks: &mut Vec<PhysicsBlock>,
    orders: &[Vec<usize>],
    snap_ids: &[usize],
) {
    let old_targets: HashMap<usize, f32> = blocks
        .iter()
        .map(|b| (b.internal_id, b.target_y))
        .collect();

    let idx_of: HashMap<usize, usize> = blocks
        .iter()
        .enumerate()
        .map(|(i, b)| (b.internal_id, i))
        .collect();

    for (page_idx, page_order) in orders.iter().enumerate() {
        let mut y = PAGE_TOP + EDGE_PAD;
        let mut prev_id: Option<usize> = None;

        for &id in page_order {
            let Some(&idx) = idx_of.get(&id) else {
                continue;
            };

            let block = &mut blocks[idx];
            block.x = block_x(page_idx);
            block.prev = prev_id;

            let new_target = y;
            let old_y = block.y;
            let old_target = old_targets
                .get(&id)
                .copied()
                .unwrap_or(block.target_y);

            if snap_ids.contains(&id) {
                block.target_y = new_target;
                block.y = new_target;
                block.vel = 0.0;
                block.phase = Phase::Idle;
            } else {
                block.target_y = new_target;

                if (old_y - new_target).abs() < 0.5 {
                    block.y = new_target;
                    block.vel = 0.0;
                    block.phase = Phase::Idle;
                } else {
                    match block.phase {
                        Phase::Idle => block.phase = Phase::Waiting,
                        Phase::Waiting => {}
                        Phase::Moving => {
                            let old_dir = (old_target - old_y).signum();
                            let new_dir = (new_target - block.y).signum();

                            if old_dir != new_dir {
                                block.vel = 0.0;
                            }
                        }
                    }
                }
            }

            y = new_target + current_height(block) + GAP;
            prev_id = Some(id);
        }
    }
}

fn should_release(block: &PhysicsBlock, snapshot: &[PhysicsBlock]) -> bool {
    match block.prev {
        None => true,
        Some(prev_id) => match snapshot.iter().find(|p| p.internal_id == prev_id) {
            None => true,
            Some(prev) => {
                if prev.phase == Phase::Idle {
                    true
                } else {
                    let gap = (block.y - (prev.y + current_height(prev))) - GAP;
                    gap.abs() >= TRIGGER
                }
            }
        },
    }
}

fn inherit_velocity(
    block: &PhysicsBlock,
    snapshot: &[PhysicsBlock],
) -> f32 {
    let Some(prev_id) = block.prev else {
        return 0.0;
    };

    let Some(prev) = snapshot.iter().find(|p| p.internal_id == prev_id) else {
        return 0.0;
    };

    if prev.phase != Phase::Moving {
        return 0.0;
    }

    let prev_dir = (prev.target_y - prev.y).signum();
    let block_dir = (block.target_y - block.y).signum();

    if prev_dir == block_dir && prev_dir != 0.0 {
        prev.vel
    } else {
        0.0
    }
}

fn release_pass(blocks: &mut [PhysicsBlock]) {
    let snapshot = blocks.to_vec();
    let mut releases = Vec::new();

    for block in &snapshot {
        if block.phase == Phase::Waiting && should_release(block, &snapshot) {
            releases.push((
                block.internal_id,
                inherit_velocity(block, &snapshot),
            ));
        }
    }

    for (id, velocity) in releases {
        if let Some(block) = blocks.iter_mut().find(|b| b.internal_id == id) {
            block.phase = Phase::Moving;
            block.vel = velocity;
        }
    }
}

fn step(blocks: &mut [PhysicsBlock]) -> bool {
    let mut did_work = false;

    for block in blocks.iter_mut() {
        if block.phase != Phase::Moving {
            continue;
        }

        did_work = true;

        let dir = (block.target_y - block.y).signum();
        block.vel = (block.vel + ACCEL * DT).min(MAX_V);
        block.y += dir * block.vel * DT;

        let reached = (dir > 0.0 && block.y >= block.target_y)
            || (dir < 0.0 && block.y <= block.target_y);

        if reached {
            block.y = block.target_y;
            block.vel = 0.0;
            block.phase = Phase::Idle;
        }
    }

    let snapshot = blocks.to_vec();
    let mut releases = Vec::new();

    for block in &snapshot {
        if block.phase == Phase::Waiting && should_release(block, &snapshot) {
            did_work = true;
            releases.push((
                block.internal_id,
                inherit_velocity(block, &snapshot),
            ));
        }
    }

    for (id, velocity) in releases {
        if let Some(block) = blocks.iter_mut().find(|b| b.internal_id == id) {
            block.phase = Phase::Moving;
            block.vel = velocity;
        }
    }

    did_work
}

fn rebuild_world(
    rapier: &RapierWorld,
    blocks: &[PhysicsBlock],
    page_count: usize,
) {
    let mut entries: Vec<(usize, (f32, f32), (f32, f32))> = blocks
        .iter()
        .map(|block| {
            (
                block.internal_id,
                (block.x, block.target_y),
                (BLOCK_W, current_height(block)),
            )
        })
        .collect();

    for page in 0..page_count {
        let x0 = page_x(page) - 16.0;
        let x1 = page_x(page) + PAGE_W + 16.0;

        entries.push((
            PAGE_ID_BASE + page,
            (x0, PAGE_TOP),
            (x1 - x0, REGION_H),
        ));
    }

    rapier.rebuild(&entries);
}

fn single_removed(current_order: &[String], incoming_order: &[String]) -> Option<String> {
    if current_order.len() != incoming_order.len() + 1 {
        return None;
    }

    for idx in 0..current_order.len() {
        let mut reduced = current_order.to_vec();
        let removed = reduced.remove(idx);

        if reduced == incoming_order {
            return Some(removed);
        }
    }

    None
}

fn fast_delete(
    removed_id: &str,
    incoming_order: &[String],
    current_boxes: &[PhysicsBlock],
    boxes: RwSignal<Vec<PhysicsBlock>>,
    id_map: RwSignal<HashMap<String, usize>>,
    drag: RwSignal<Option<DragState>>,
    rapier: &RapierWorld,
) -> bool {
    if !current_boxes.iter().any(|b| b.id == removed_id) {
        return false;
    }

    let mut new_boxes: Vec<PhysicsBlock> = current_boxes
        .iter()
        .filter(|b| b.id != removed_id)
        .cloned()
        .collect();

    let new_map: HashMap<String, usize> = new_boxes
        .iter()
        .map(|b| (b.id.clone(), b.internal_id))
        .collect();

    let order: Vec<usize> = incoming_order
        .iter()
        .filter_map(|id| new_map.get(id).copied())
        .collect();

    if order.len() != incoming_order.len() {
        return false;
    }

    commit(&mut new_boxes, &[order], &[]);
    release_pass(&mut new_boxes);

    boxes.set(new_boxes.clone());
    id_map.set(new_map);
    rebuild_world(rapier, &new_boxes, 1);
    drag.set(None);

    true
}

fn fast_add(
    added_id: &str,
    incoming: &[BlockData],
    incoming_order: &[String],
    current_boxes: &[PhysicsBlock],
    boxes: RwSignal<Vec<PhysicsBlock>>,
    next_internal_id: RwSignal<usize>,
    id_map: RwSignal<HashMap<String, usize>>,
    drag: RwSignal<Option<DragState>>,
    rapier: &RapierWorld,
) -> bool {
    let Some(added_row) = incoming.iter().find(|b| b.id == added_id) else {
        return false;
    };

    if current_boxes.iter().any(|b| b.id == added_id) {
        return false;
    }

    let internal_id = next_internal_id.get_untracked();
    let mut new_boxes = current_boxes.to_vec();

    new_boxes.push(PhysicsBlock {
        internal_id,
        id: added_row.id.clone(),
        content: ArcRwSignal::new(added_row.content.clone()),
        x: 0.0,
        y: 0.0,
        target_y: 0.0,
        vel: 0.0,
        h: UNMEASURED_HEIGHT,
        phase: Phase::Idle,
        prev: None,
        measured: false,
    });

    let new_map: HashMap<String, usize> = new_boxes
        .iter()
        .map(|b| (b.id.clone(), b.internal_id))
        .collect();

    let order: Vec<usize> = incoming_order
        .iter()
        .filter_map(|id| new_map.get(id).copied())
        .collect();

    if order.len() != incoming_order.len() {
        return false;
    }

    next_internal_id.set(internal_id + 1);

    commit(&mut new_boxes, &[order], &[internal_id]);
    release_pass(&mut new_boxes);

    boxes.set(new_boxes.clone());
    id_map.set(new_map);
    rebuild_world(rapier, &new_boxes, 1);
    drag.set(None);

    true
}

fn sync_from_props(
    incoming: Vec<BlockData>,
    boxes: RwSignal<Vec<PhysicsBlock>>,
    next_internal_id: RwSignal<usize>,
    id_map: RwSignal<HashMap<String, usize>>,
    drag: RwSignal<Option<DragState>>,
    rapier: &RapierWorld,
) {
    let current_boxes = boxes.get_untracked();

    // Keep the internal content signals synchronized with parent data.
    for incoming_block in &incoming {
        if let Some(existing) = current_boxes.iter().find(|b| b.id == incoming_block.id) {
            if existing.content.get_untracked() != incoming_block.content {
                existing.content.set(incoming_block.content.clone());
            }
        }
    }

    let incoming_order: Vec<String> =
        incoming.iter().map(|b| b.id.clone()).collect();

    let current_order: Vec<String> = rapier
        .page_order(0)
        .into_iter()
        .filter_map(|internal_id| {
            current_boxes
                .iter()
                .find(|b| b.internal_id == internal_id)
                .map(|b| b.id.clone())
        })
        .collect();

    // If the current rendered order already matches the parent,
    // no structural synchronization is needed.
    if incoming_order == current_order
        && incoming_order.len() == current_boxes.len()
    {
        return;
    }

    // Pure delete: exactly one block disappeared, with the other
    // blocks retaining their relative order.
    if let Some(removed_id) = single_removed(&current_order, &incoming_order) {
        if fast_delete(
            &removed_id,
            &incoming_order,
            &current_boxes,
            boxes,
            id_map,
            drag,
            rapier,
        ) {
            return;
        }
    }

    // Pure add: exactly one block appeared, with the existing blocks
    // retaining their relative order.
    if let Some(added_id) = single_removed(&incoming_order, &current_order) {
        if fast_add(
            &added_id,
            &incoming,
            &incoming_order,
            &current_boxes,
            boxes,
            next_internal_id,
            id_map,
            drag,
            rapier,
        ) {
            return;
        }
    }

    // General rebuild: preserve internal IDs and content signals for
    // existing blocks; mint IDs only for blocks not already present.
    let mut current_map = id_map.get_untracked();

    for block in &current_boxes {
        current_map
            .entry(block.id.clone())
            .or_insert(block.internal_id);
    }

    let mut new_map = HashMap::new();
    let mut new_boxes = Vec::with_capacity(incoming.len());

    for incoming_block in &incoming {
        let internal_id = if let Some(&existing_id) = current_map.get(&incoming_block.id) {
            existing_id
        } else {
            let new_id = next_internal_id.get_untracked();
            next_internal_id.set(new_id + 1);
            new_id
        };

        new_map.insert(incoming_block.id.clone(), internal_id);

        if let Some(existing) = current_boxes
            .iter()
            .find(|b| b.internal_id == internal_id)
        {
            new_boxes.push(existing.clone());
        } else {
            new_boxes.push(PhysicsBlock {
                internal_id,
                id: incoming_block.id.clone(),
                content: ArcRwSignal::new(incoming_block.content.clone()),
                x: 0.0,
                y: 0.0,
                target_y: 0.0,
                vel: 0.0,
                h: UNMEASURED_HEIGHT,
                phase: Phase::Idle,
                prev: None,
                measured: false,
            });
        }
    }

    let order: Vec<usize> = incoming_order
        .iter()
        .filter_map(|id| new_map.get(id).copied())
        .collect();

    let snap_ids: Vec<usize> =
        new_boxes.iter().map(|b| b.internal_id).collect();

    commit(&mut new_boxes, &[order], &snap_ids);
    release_pass(&mut new_boxes);

    boxes.set(new_boxes.clone());
    id_map.set(new_map);
    rebuild_world(rapier, &new_boxes, 1);
    drag.set(None);
}

fn apply_height(
    internal_id: usize,
    height: f32,
    boxes: RwSignal<Vec<PhysicsBlock>>,
    rapier: &RapierWorld,
) {
    if height < 1.0 {
        return;
    }

    let mut current = boxes.get_untracked();
    let mut changed = false;

    if let Some(block) = current
        .iter_mut()
        .find(|b| b.internal_id == internal_id)
    {
        if !block.measured || (block.h - height).abs() >= 0.5 {
            changed = true;
        }

        block.h = height;
        block.measured = true;
    }

    if !changed {
        return;
    }

    let mut order = rapier.page_order(0);

    if order.is_empty() {
        order = current.iter().map(|b| b.internal_id).collect();
    }

    let snap_ids: Vec<usize> =
        current.iter().map(|b| b.internal_id).collect();

    commit(&mut current, &[order], &snap_ids);
    boxes.set(current.clone());
    rebuild_world(rapier, &current, 1);
}

#[component]
fn App() -> impl IntoView {
    let blocks = RwSignal::new(vec![
        BlockData { id: "a".into(), content: String::new() },
        BlockData { id: "b".into(), content: String::new() },
        BlockData { id: "c".into(), content: String::new() },
    ]);
    let next_id = RwSignal::new(1usize);

    let on_split = Callback::new(move |(block_id, after): (String, String)| {
        let blocks = blocks;
        let next_id = next_id;

        spawn_local(async move {
            TimeoutFuture::new(50).await;

            let mut refreshed_rows = blocks.get_untracked();
            let mut new_id_for_focus = None;

            if let Some(idx) = refreshed_rows.iter().position(|b| b.id == block_id) {
                let new_id = format!("b-{}", next_id.get_untracked());
                next_id.update(|n| *n += 1);

                refreshed_rows.insert(
                    idx + 1,
                    BlockData {
                        id: new_id.clone(),
                        content: after,
                    },
                );

                new_id_for_focus = Some(new_id);
            }

            if let Some(id) = new_id_for_focus {
                blocks.set(refreshed_rows);
                FOCUS_BLOCK_AT_START.set(Some(id));
            }
        });
    });

    let on_content_change = Callback::new(move |(block_id, new_text): (String, String)| {
        blocks.update(|list| {
            if let Some(block) = list.iter_mut().find(|b| b.id == block_id) {
                block.content = new_text;
            }
        });
    });

    let on_split_for_input = on_split.clone();
    let on_content_change_for_input = on_content_change.clone();

    let on_block_input = Callback::new(move |evt: BlockInputEvent| {
        let BlockInputEvent {
            block_id,
            old_text,
            new_text,
            was_enter,
            cursor_pos,
        } = evt;

        let utf16: Vec<u16> = new_text.encode_utf16().collect();

        let is_split = was_enter
            && !old_text.is_empty()
            && cursor_pos >= 1
            && cursor_pos <= utf16.len()
            && utf16[cursor_pos - 1] == b'\n' as u16;

        if is_split {
            let before = String::from_utf16_lossy(&utf16[..cursor_pos - 1]);
            let after = String::from_utf16_lossy(&utf16[cursor_pos..]);

            on_content_change_for_input.run((block_id.clone(), before));
            on_split_for_input.run((block_id, after));
        } else {
            on_content_change_for_input.run((block_id, new_text));
        }
    });

    let blocks_for_reorder = blocks;
    let on_reorder = Callback::new(
        move |(block_id, above_id): (String, Option<String>)| {
            let mut rows = blocks_for_reorder.get_untracked();

            let Some(moved_idx) = rows.iter().position(|b| b.id == block_id) else {
                return;
            };

            let moved = rows.remove(moved_idx);
            let insert_idx = above_id
                .as_ref()
                .and_then(|id| rows.iter().position(|b| &b.id == id))
                .map(|idx| idx + 1)
                .unwrap_or(0);

            rows.insert(insert_idx, moved);
            blocks_for_reorder.set(rows);
        },
    );

    let blocks_for_delete = blocks;
    let on_delete_request = Callback::new(move |block_id: String| {
        let rows = blocks_for_delete.get_untracked();

        if rows.len() <= 1 {
            return;
        }

        let above_id = rows
            .iter()
            .position(|b| b.id == block_id)
            .and_then(|idx| idx.checked_sub(1))
            .and_then(|idx| rows.get(idx))
            .map(|b| b.id.clone());

        blocks_for_delete.update(|list| {
            list.retain(|b| b.id != block_id);
        });

        if let Some(id) = above_id {
            FOCUS_BLOCK_ID.set(Some(id));
        }
    });

    let blocks_for_merge = blocks;
    let on_merge_with_above =
        Callback::new(move |(block_id, current_text): (String, String)| {
            let rows = blocks_for_merge.get_untracked();

            let Some(idx) = rows.iter().position(|b| b.id == block_id) else {
                return;
            };

            let Some(above_idx) = idx.checked_sub(1) else {
                return;
            };

            let Some(above) = rows.get(above_idx) else {
                return;
            };

            let above_id = above.id.clone();
            let above_content = above.content.clone();

            if above_content.is_empty() {
                blocks_for_merge.update(|list| {
                    list.retain(|b| b.id != above_id);
                });
                return;
            }

            let new_above = format!("{}{}", above_content, current_text);

            blocks_for_merge.update(|list| {
                list.retain(|b| b.id != block_id);

                if let Some(block) = list.iter_mut().find(|b| b.id == above_id) {
                    block.content = new_above.clone();
                }
            });

            FOCUS_BLOCK_ID.set(Some(above_id));
        });

    let on_block_input_for_render = on_block_input.clone();
    let on_delete_for_render = on_delete_request.clone();
    let on_merge_for_render = on_merge_with_above.clone();

    let render_block: Callback<BlockRenderInput, AnyView> =
        Callback::new(move |input: BlockRenderInput| {
            let BlockRenderInput {
                block_id,
                content,
                internal_id,
                boxes,
                rapier,
            } = input;

            let on_input = {
                let callback = on_block_input_for_render.clone();
                Callback::new(move |evt: BlockInputEvent| callback.run(evt))
            };

            let on_delete = {
                let callback = on_delete_for_render.clone();
                Callback::new(move |id: String| callback.run(id))
            };

            let on_merge = {
                let callback = on_merge_for_render.clone();
                Callback::new(move |args: (String, String)| callback.run(args))
            };

            view! {
                <BlockItem
                    block_id=block_id
                    content=content
                    internal_id=internal_id
                    on_block_input=on_input
                    on_delete_request=on_delete
                    on_merge_with_above=on_merge
                    boxes=boxes
                    rapier=rapier
                />
            }
            .into_any()
        });

    view! {
        <style>{".ps-unmeasured { visibility: hidden; }"}</style>
        <div style="padding: 20px; font-family: monospace;">
            <div style="margin-bottom: 12px;">
                "Rapier layout test: Enter splits; Backspace at the start merges."
            </div>

            <BlockContainer
                blocks=Signal::derive(move || blocks.get())
                render_block=render_block
                on_reorder=on_reorder
            />
        </div>
    }
}

#[component]
fn BlockContainer(
    #[prop(into)] blocks: Signal<Vec<BlockData>>,
    render_block: Callback<BlockRenderInput, AnyView>,
    on_reorder: Callback<(String, Option<String>)>,
) -> impl IntoView {
    let boxes = RwSignal::new(Vec::<PhysicsBlock>::new());
    let next_internal_id = RwSignal::new(1usize);
    let id_map = RwSignal::new(HashMap::<String, usize>::new());
    let rapier = RapierWorld::new();
    let rapier_stored: StoredValue<RapierWorld, LocalStorage> =
        StoredValue::new_local(rapier.clone());
    let drag: RwSignal<Option<DragState>> = RwSignal::new(None);
    let stage_ref = NodeRef::<Div>::new();

    let observer_stored: StoredValue<Option<web_sys::ResizeObserver>, LocalStorage> =
        StoredValue::new_local(None);

    {
        let boxes_obs = boxes;
        let rapier_obs = rapier.clone();

        let cb = Closure::<dyn FnMut(js_sys::Array, web_sys::ResizeObserver)>::new(
            move |entries: js_sys::Array, _obs: web_sys::ResizeObserver| {
                let mut pairs: Vec<(usize, f32)> = Vec::new();

                for i in 0..entries.length() {
                    let entry_val = entries.get(i);
                    let entry: web_sys::ResizeObserverEntry = entry_val.unchecked_into();
                    let target = entry.target();
                    let html: web_sys::HtmlElement = target.unchecked_into();
                    let height = html.offset_height() as f32;

                    if height < 1.0 {
                        continue;
                    }

                    let Some(id_attr) = html.get_attribute("data-id") else {
                        continue;
                    };
                    let Ok(id) = id_attr.parse::<usize>() else {
                        continue;
                    };

                    pairs.push((id, height));
                }

                for (id, height) in pairs {
                    apply_height(id, height, boxes_obs, &rapier_obs);
                }
            },
        );

        let observer = web_sys::ResizeObserver::new(cb.as_ref().unchecked_ref())
            .expect("ResizeObserver::new");
        cb.forget();
        observer_stored.set_value(Some(observer));
    }

    {
        let rapier_for_effect = rapier.clone();

        Effect::new(move |_| {
            sync_from_props(
                blocks.get(),
                boxes,
                next_internal_id,
                id_map,
                drag,
                &rapier_for_effect,
            );
        });
    }

    let boxes_for_interval = boxes;

    let interval_handle = set_interval_with_handle(
        move || {
            let mut current = boxes_for_interval.get_untracked();

            if step(&mut current) {
                boxes_for_interval.set(current);
            }
        },
        Duration::from_millis(16),
    )
    .expect("failed to set interval");

    let drag_for_move = drag;
    let stage_for_move = stage_ref;

    let on_mouse_move = move |e: ev::MouseEvent| {
        let Some(mut state) = drag_for_move.get_untracked() else {
            return;
        };

        let Some(stage) = stage_for_move.get() else {
            return;
        };

        let rect = stage.get_bounding_client_rect();
        let mx = e.client_x() as f32 - rect.left() as f32;
        let my = e.client_y() as f32 - rect.top() as f32;

        state.current = (mx - state.offset.0, my - state.offset.1);

        if (mx - state.start.0).abs() > DRAG_THRESHOLD
            || (my - state.start.1).abs() > DRAG_THRESHOLD
        {
            state.moved = true;
        }

        drag_for_move.set(Some(state));
    };

    let mousemove_handle = window_event_listener(ev::mousemove, on_mouse_move);

    let drag_for_up = drag;
    let stage_for_up = stage_ref;
    let boxes_for_up = boxes;
    let rapier_for_up = rapier_stored;
    let on_reorder_for_up = on_reorder;

    let on_mouse_up = move |_e: ev::MouseEvent| {
        let Some(state) = drag_for_up.get_untracked() else {
            return;
        };

        drag_for_up.set(None);

        if !state.moved {
            return;
        }

        let Some(_stage) = stage_for_up.get() else {
            return;
        };

        let current = boxes_for_up.get_untracked();

        let Some(moved_block) = current
            .iter()
            .find(|b| b.internal_id == state.id)
            .cloned()
        else {
            return;
        };

        let point = (
            state.current.0 + BLOCK_W / 2.0,
            state.current.1 + current_height(&moved_block) / 2.0,
        );

        let drop_target = rapier_for_up.with_value(|world| {
            world.locate_drop(point, &[state.id])
        });

        let Some((page, above_internal_id)) = drop_target else {
            return;
        };

        if page != 0 {
            return;
        }

        rapier_for_up.with_value(|world| {
            apply_move(
                state.id,
                page,
                above_internal_id,
                boxes_for_up,
                world,
                on_reorder_for_up.clone(),
            );
        });
    };

    let mouseup_handle = window_event_listener(ev::mouseup, on_mouse_up);

    on_cleanup(move || {
        interval_handle.clear();
        mousemove_handle.remove();
        mouseup_handle.remove();

        observer_stored.with_value(|observer| {
            if let Some(observer) = observer {
                observer.disconnect();
            }
        });
    });



    view! {
        <div
            node_ref=stage_ref
            style=move || format!(
                "position:relative;width:{}px;height:{}px;",
                stage_width(),
                stage_height(&boxes.get()),
            )
            on:mouseleave=move |_| {
                drag.set(None);
            }
        >
            <div
                style=move || format!(
                    "position:absolute;left:{}px;top:{}px;width:{}px;height:{}px;\
                     box-sizing:border-box;border:1px solid #dedede;\
                     background:#fafafa;padding:8px;color:#777;",
                    page_x(0),
                    PAGE_TOP,
                    PAGE_W,
                    page_panel_height(stage_height(&boxes.get())),
                )
            >
                "TEST PAGE"
            </div>

            <For
                each=move || boxes.get()
                key=|block| block.internal_id
                children=move |block: PhysicsBlock| {
                    let internal_id = block.internal_id;
                    let block_render_input = BlockRenderInput {
                        block_id: block.id.clone(),
                        content: block.content.clone(),
                        internal_id,
                        boxes,
                        rapier: rapier_stored,
                    };
                    let node_ref = NodeRef::<Div>::new();
                    let observer_for_mount = observer_stored;
                    let drag_for_style = drag;

                    Effect::new(move |_| {
                        let Some(el) = node_ref.get() else {
                            return;
                        };
                        spawn_local(async move {
                            TimeoutFuture::new(2000).await;
                            observer_for_mount.with_value(|observer| {
                                if let Some(observer) = observer {
                                    let _ = observer.observe(&el);
                                }
                            });
                        });
                    });

                    let style_now = move || {
                        let current = boxes.get();
                        let found = current.iter().find(|b| b.internal_id == internal_id);

                        let mut x = found.map_or(0.0, |b| b.x);
                        let mut y = found.map_or(0.0, |b| b.y);

                        if let Some(state) = drag_for_style.get() {
                            if state.id == internal_id {
                                x = state.current.0;
                                y = state.current.1;
                            }
                        }

                        format!(
                            "position:absolute;left:{}px;top:{}px;width:{}px;",
                            x, y, BLOCK_W,
                        )
                    };

                    view! {
                        <div
                            node_ref=node_ref
                            style=style_now
                            class:ps-unmeasured=move || {
                                boxes
                                    .get()
                                    .iter()
                                    .find(|b| b.internal_id == internal_id)
                                    .map_or(true, |b| !b.measured)
                            }
                            data-id=internal_id
                        >
                            <div
                                class="ps-handle"
                                style="position:absolute;left:-30px;top:2px;\
                                       cursor:grab;user-select:none;padding:2px 4px;\
                                       color:#777;"
                                on:mousedown=move |e: ev::MouseEvent| {
                                    let Some(stage) = stage_ref.get() else {
                                        return;
                                    };

                                    let rect = stage.get_bounding_client_rect();
                                    let mx = e.client_x() as f32 - rect.left() as f32;
                                    let my = e.client_y() as f32 - rect.top() as f32;

                                    let live = boxes
                                        .get_untracked()
                                        .iter()
                                        .find(|b| b.internal_id == internal_id)
                                        .map(|b| (b.x, b.y))
                                        .unwrap_or((0.0, 0.0));

                                    drag.set(Some(DragState {
                                        id: internal_id,
                                        offset: (mx - live.0, my - live.1),
                                        current: live,
                                        moved: false,
                                        start: (mx, my),
                                    }));

                                    e.prevent_default();
                                }
                            >
                                "::"
                            </div>

                            {render_block.run(block_render_input)}
                        </div>
                    }
                }
            />
        </div>
    }
}

#[component]
fn BlockItem(
    block_id: String,
    content: ArcRwSignal<String>,
    internal_id: usize,
    on_block_input: Callback<BlockInputEvent>,
    on_delete_request: Callback<String>,
    on_merge_with_above: Callback<(String, String)>,
    boxes: RwSignal<Vec<PhysicsBlock>>,
    rapier: StoredValue<RapierWorld, LocalStorage>,
) -> impl IntoView {
    let textarea_ref = NodeRef::<leptos::html::Textarea>::new();
    let initial = content.get_untracked();

    let my_id_for_focus = block_id.clone();
    let my_id_for_end_focus = block_id.clone();
    let my_id_for_input = block_id.clone();
    let my_id_for_keydown = block_id.clone();

    let content_signal = content.clone();
    let content_signal_for_render = content_signal.clone();
    let content_signal_for_focus = content_signal.clone();
    let content_signal_for_end_focus = content_signal.clone();
    let content_signal_for_keydown = content_signal.clone();

    let boxes_for_render = boxes;
    let rapier_for_render = rapier.clone();
    let boxes_for_mount = boxes;
    let rapier_for_mount = rapier.clone();
    let boxes_for_input = boxes;
    let rapier_for_input = rapier.clone();


    let on_delete_for_keydown = on_delete_request.clone();
    let on_merge_for_keydown = on_merge_with_above.clone();

    Effect::new(move |_| {
        let current = content_signal_for_render.get();

        if let Some(el) = textarea_ref.get_untracked() {
            if el.value() != current {
                el.set_value(&current);
                let _ = web_sys::HtmlElement::style(&el).set_property("height", "auto");

                let sh = el.scroll_height();

                let _ = web_sys::HtmlElement::style(&el)
                    .set_property("height", &format!("{}px", sh));
            }

            let visible = web_sys::window()
                .and_then(|w| w.get_computed_style(el.unchecked_ref::<web_sys::Element>()).ok().flatten())
                .map(|cs| cs.get_property_value("visibility").ok() == Some("visible".to_string()))
                .unwrap_or(false);

            if visible {
                let height = el.offset_height() as f32;

                if height > 0.0 {
                    rapier_for_render.with_value(|world| {
                        apply_height(internal_id, height, boxes_for_render, world);
                    });
                }
            }
        }
    });

    // Measure again over a few frames, as the real component does
    // while the browser finishes laying out the textarea.
    Effect::new(move |_| {
        let rapier_for_mount = rapier_for_mount.clone();
        spawn_local(async move {
            for _ in 0..3 {
                TimeoutFuture::new(2000).await;

                if let Some(el) = textarea_ref.get_untracked() {
                    let _ = web_sys::HtmlElement::style(&el).set_property("height", "auto");

                    let sh = el.scroll_height();

                    let _ = web_sys::HtmlElement::style(&el)
                        .set_property("height", &format!("{}px", sh));

                    let height = el.offset_height() as f32;

                    if height > 0.0 {
                        rapier_for_mount.with_value(|world| {
                            apply_height(internal_id, height, boxes_for_mount, world);
                        });
                    }
                }
            }
        });
    });

    Effect::new(move |_| {
        let Some(target) = FOCUS_BLOCK_AT_START.get() else {
            return;
        };

        if target != my_id_for_focus {
            return;
        }

        let content_sig = content_signal_for_focus.clone();
        let focus_id = my_id_for_focus.clone();

        spawn_local(async move {
            for _ in 0..20 {
                if let Some(el) = textarea_ref.get_untracked() {
                    let current = content_sig.get_untracked();

                    if el.value() != current {
                        el.set_value(&current);
                    }

                    let _ = el.focus();
                    let _ = el.set_selection_range(0, 0);

                    let visibility = web_sys::window()
                        .and_then(|window| window.get_computed_style(&el).ok().flatten())
                        .and_then(|style| style.get_property_value("visibility").ok())
                        .unwrap_or_else(|| "<unknown>".to_string());

                    let active_id = web_sys::window()
                        .and_then(|window| window.document())
                        .and_then(|document| document.active_element())
                        .and_then(|active| active.get_attribute("data-block-id"))
                        .unwrap_or_else(|| "<none>".to_string());

                    web_sys::console::log_1(
                        &format!(
                            "focus target={} active={} visibility={}",
                            focus_id, active_id, visibility
                        )
                        .into(),
                    );

                    FOCUS_BLOCK_AT_START.set(None);
                    return;
                }

                TimeoutFuture::new(16).await;
            }

            FOCUS_BLOCK_AT_START.set(None);
        });
    });

    Effect::new(move |_| {
        let Some(target) = FOCUS_BLOCK_ID.get() else {
            return;
        };

        if target != my_id_for_end_focus {
            return;
        }

        if let Some(el) = textarea_ref.get_untracked() {
            let current = content_signal_for_end_focus.get_untracked();

            if el.value() != current {
                el.set_value(&current);
            }

            let _ = el.focus();
            let pos = current.encode_utf16().count() as u32;
            let _ = el.set_selection_range(pos, pos);
        }

        FOCUS_BLOCK_ID.set(None);
    });

    let was_enter = StoredValue::new(false);

    let on_keydown = move |e: ev::KeyboardEvent| {
        if e.key() == "Enter" {
            was_enter.set_value(true);
            return;
        }

        let is_empty = content_signal_for_keydown.get_untracked().is_empty();

        if (e.key() == "Backspace" || e.key() == "Delete") && is_empty {
            on_delete_for_keydown.run(my_id_for_keydown.clone());
            return;
        }

        if e.key() == "Backspace" && !is_empty {
            if let Some(el) = textarea_ref.get_untracked() {
                let start = el.selection_start().ok().flatten().unwrap_or(1);
                let end = el.selection_end().ok().flatten().unwrap_or(1);

                if start == 0 && end == 0 {
                    on_merge_for_keydown.run((
                        my_id_for_keydown.clone(),
                        el.value(),
                    ));
                }
            }
        }
    };

    let on_input = move |e: ev::Event| {
        let Some(target) = e.target() else {
            return;
        };

        let textarea: web_sys::HtmlTextAreaElement = target.unchecked_into();
        let new_value = textarea.value();

        let was_enter_val = was_enter.get_value();
        was_enter.set_value(false);

        let cursor_pos = textarea
            .selection_start()
            .ok()
            .flatten()
            .unwrap_or(0) as usize;

        let old_text = content_signal.get_untracked();
        content_signal.set(new_value.clone());

        on_block_input.run(BlockInputEvent {
            block_id: my_id_for_input.clone(),
            old_text,
            new_text: new_value,
            was_enter: was_enter_val,
            cursor_pos,
        });

        let _ = web_sys::HtmlElement::style(&textarea).set_property("height", "auto");

        let sh = textarea.scroll_height();

        let _ = web_sys::HtmlElement::style(&textarea)
            .set_property("height", &format!("{}px", sh));

        let height = textarea.offset_height() as f32;

        if height > 0.0 {
            rapier_for_input.with_value(|world| {
                apply_height(internal_id, height, boxes_for_input, world);
            });
        }
    };

    view! {
        <textarea
            node_ref=textarea_ref
            data-block-id=block_id.clone()
            prop:value=initial
            rows="1"
            style="width:100%;display:block;box-sizing:border-box;margin:0;\
                   resize:none;overflow:hidden;outline:none;\
                   border:1px solid rgba(0,0,0,0.05);"
            on:keydown=on_keydown
            on:input=on_input
        ></textarea>
    }
}

fn apply_move(
    moved_id: usize,
    dest_page: usize,
    above: Option<usize>,
    boxes: RwSignal<Vec<PhysicsBlock>>,
    rapier: &RapierWorld,
    on_reorder: Callback<(String, Option<String>)>,
) {
    if dest_page != 0 || !rapier.page_order(0).contains(&moved_id) {
        return;
    }

    let mut order: Vec<usize> = rapier
        .page_order(0)
        .into_iter()
        .filter(|&id| id != moved_id)
        .collect();

    let insert_at = match above {
        None => 0,
        Some(above_id) => order
            .iter()
            .position(|&id| id == above_id)
            .map(|idx| idx + 1)
            .unwrap_or(order.len()),
    };

    order.insert(insert_at, moved_id);

    let mut current = boxes.get_untracked();
    commit(&mut current, &[order.clone()], &[moved_id]);
    release_pass(&mut current);
    boxes.set(current.clone());
    rebuild_world(rapier, &current, 1);

    let Some(position) = order.iter().position(|&id| id == moved_id) else {
        return;
    };

    let above_id = position
        .checked_sub(1)
        .and_then(|idx| order.get(idx))
        .and_then(|id| current.iter().find(|b| b.internal_id == *id))
        .map(|b| b.id.clone());

    let Some(moved_block_id) = current
        .iter()
        .find(|b| b.internal_id == moved_id)
        .map(|b| b.id.clone())
    else {
        return;
    };

    on_reorder.run((moved_block_id, above_id));
}
