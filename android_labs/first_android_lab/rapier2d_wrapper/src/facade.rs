use std::collections::HashMap;
use std::sync::Mutex;

use crate::core::Core;
use crate::types::{Config, Mode};

#[derive(uniffi::Record)]
pub struct StepResult {
    pub moving: bool,
    pub changed: Vec<BlockPos>,
}

#[derive(uniffi::Record)]
pub struct BlockPos {
    pub id: u64,
    pub y: f32,
}

#[derive(uniffi::Record)]
pub struct PageRect {
    pub page: u32,
    pub y: f32,
    pub h: f32,
}

#[derive(uniffi::Record)]
pub struct DropTarget {
    pub page: u32,
    pub above: Option<u64>,
}

#[derive(uniffi::Object)]
pub struct CascadePhysics {
    inner: Mutex<Inner>,
}

struct Inner {
    core: Core,
    last_y: HashMap<u64, f32>,
}

#[uniffi::export]
impl CascadePhysics {
    #[uniffi::constructor]
    pub fn new(config: Config) -> Self {
        Self {
            inner: Mutex::new(Inner {
                core: Core::new(config),
                last_y: HashMap::new(),
            }),
        }
    }

    pub fn set_mode(&self, mode: Mode) {
        self.inner.lock().unwrap().core.set_mode(mode);
    }

    pub fn set_reduced(&self, reduced: bool) {
        self.inner.lock().unwrap().core.set_reduced(reduced);
    }

    pub fn add_block(&self, id: u64, page: u32, est_height: f32) {
        let mut g = self.inner.lock().unwrap();
        g.core.add_block(id, page as usize, est_height);
        g.last_y.insert(id, 0.0);
    }

    pub fn remove_block(&self, id: u64) {
        let mut g = self.inner.lock().unwrap();
        g.core.remove_block(id);
        g.last_y.remove(&id);
    }

    pub fn set_height(&self, id: u64, h: f32) -> Vec<BlockPos> {
        let mut g = self.inner.lock().unwrap();
        g.core.set_height(id, h);
        absorb_changes(&mut g)
    }

    pub fn commit_order(&self, orders: Vec<Vec<u64>>, snap_ids: Vec<u64>) -> Vec<BlockPos> {
        let mut g = self.inner.lock().unwrap();
        g.core.commit_order(&orders, &snap_ids);
        absorb_changes(&mut g)
    }

    pub fn step(&self, dt: f32) -> StepResult {
        let mut g = self.inner.lock().unwrap();
        let moving = g.core.step(dt);
        let changed = absorb_changes(&mut g);
        StepResult { moving, changed }
    }

    pub fn page_rects(&self) -> Vec<PageRect> {
        self.inner
            .lock()
            .unwrap()
            .core
            .page_rects()
            .into_iter()
            .map(|(p, y, h)| PageRect {
                page: p as u32,
                y,
                h,
            })
            .collect()
    }

    pub fn page_order(&self, page: u32) -> Vec<u64> {
        self.inner.lock().unwrap().core.page_order(page as usize)
    }

    pub fn locate_drop(&self, x: f32, y: f32, exclude: Vec<u64>) -> Option<DropTarget> {
        self.inner
            .lock()
            .unwrap()
            .core
            .locate_drop(x, y, &exclude)
            .map(|(page, above)| DropTarget {
                page: page as u32,
                above,
            })
    }
}

fn absorb_changes(g: &mut Inner) -> Vec<BlockPos> {
    let mut out = Vec::new();
    for b in g.core.blocks() {
        let prev = g.last_y.get(&b.id).copied();
        let should_report = match prev {
            None => true,
            Some(p) => (p - b.y).abs() > 0.0001,
        };
        if should_report {
            out.push(BlockPos { id: b.id, y: b.y });
            g.last_y.insert(b.id, b.y);
        }
    }
    out
}
