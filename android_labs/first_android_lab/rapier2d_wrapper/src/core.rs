use std::collections::HashMap;

use crate::integrators;
use crate::layout;
use crate::types::{
    Block, Config, Mode, Phase, RIPPLE_STAGGER_MS,
};

pub struct Core {
    config: Config,
    mode: Mode,
    reduced: bool,
    blocks: Vec<Block>,
    page_count: usize,
}

impl Core {
    pub fn new(config: Config) -> Self {
        Self {
            mode: config.mode,
            config,
            reduced: false,
            blocks: Vec::new(),
            page_count: 0,
        }
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
        self.config.mode = mode;
    }

    pub fn set_reduced(&mut self, reduced: bool) {
        self.reduced = reduced;
    }

    pub fn add_block(&mut self, id: u64, page: usize, est_height: f32) {
        if page + 1 > self.page_count {
            self.page_count = page + 1;
        }
        self.blocks.push(Block {
            id,
            page,
            y: 0.0,
            target_y: 0.0,
            h: est_height,
            vel: 0.0,
            phase: Phase::Idle,
            prev: None,
            delay: 0,
        });
    }

    pub fn remove_block(&mut self, id: u64) {
        self.blocks.retain(|b| b.id != id);
    }

    pub fn commit_order(&mut self, orders: &[Vec<u64>], snap_ids: &[u64]) {
        if orders.len() > self.page_count {
            self.page_count = orders.len();
        }
        let rects = layout::page_rects(&self.blocks, self.page_count, &self.config);
        let mode = self.mode;
        let reduced = self.reduced;

        let old_targets: HashMap<u64, f32> =
            self.blocks.iter().map(|b| (b.id, b.target_y)).collect();

        let mut id_to_idx: HashMap<u64, usize> = HashMap::new();
        for (i, b) in self.blocks.iter().enumerate() {
            id_to_idx.insert(b.id, i);
        }

        let mut rank: u32 = 0;

        for (page, order) in orders.iter().enumerate() {
            let page_top = rects[page].1;
            let mut y = page_top + self.config.top_pad;
            let mut prev_id: Option<u64> = None;
            for &id in order {
                let Some(&idx) = id_to_idx.get(&id) else { continue };
                let b = &mut self.blocks[idx];
                b.page = page;
                b.prev = prev_id;
                let new_target = y;
                let old_y = b.y;
                let old_target = old_targets.get(&id).copied().unwrap_or(b.target_y);
                b.target_y = new_target;

                let is_snap = reduced || snap_ids.contains(&id);
                let diff = (old_y - new_target).abs();

                if is_snap || diff < 0.5 {
                    b.y = new_target;
                    b.vel = 0.0;
                    b.phase = Phase::Idle;
                    b.delay = 0;
                } else {
                    match b.phase {
                        Phase::Idle => {
                            b.phase = Phase::Waiting;
                            b.delay = if mode == Mode::Ripple {
                                rank * RIPPLE_STAGGER_MS
                            } else {
                                0
                            };
                        }
                        Phase::Waiting => {}
                        Phase::Moving => {
                            let old_dir = (old_target - old_y).signum();
                            let new_dir = (new_target - b.y).signum();
                            if old_dir != new_dir {
                                b.vel = 0.0;
                            }
                        }
                    }
                    rank = rank.wrapping_add(1);
                }

                y = new_target + b.h + self.config.block_gap;
                prev_id = Some(id);
            }
        }
    }

    pub fn set_height(&mut self, id: u64, h: f32) {
        let Some(b) = self.blocks.iter_mut().find(|b| b.id == id) else {
            return;
        };
        if (b.h - h).abs() < 0.5 {
            return;
        }
        b.h = h;
        let page = b.page;

        let mut on_page: Vec<(f32, usize, u64)> = self
            .blocks
            .iter()
            .enumerate()
            .filter(|(_, b)| b.page == page)
            .map(|(i, b)| (b.target_y, i, b.id))
            .collect();
        on_page.sort_by(|a, b| {
            a.0.partial_cmp(&b.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.1.cmp(&b.1))
        });
        let Some(pos) = on_page.iter().position(|(_, _, bid)| *bid == id) else {
            return;
        };
        let snap_ids: Vec<u64> = on_page[pos + 1..]
            .iter()
            .filter(|(_, _, bid)| {
                self.blocks
                    .iter()
                    .find(|b| b.id == *bid)
                    .map(|b| b.phase == Phase::Idle)
                    .unwrap_or(false)
            })
            .map(|(_, _, bid)| *bid)
            .collect();

        let mut per_page: Vec<Vec<(f32, usize, u64)>> =
            (0..self.page_count).map(|_| Vec::new()).collect();
        for (i, b) in self.blocks.iter().enumerate() {
            if b.page < self.page_count {
                per_page[b.page].push((b.target_y, i, b.id));
            }
        }
        let mut orders: Vec<Vec<u64>> = Vec::with_capacity(self.page_count);
        for mut v in per_page {
            v.sort_by(|a, b| {
                a.0.partial_cmp(&b.0)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then(a.1.cmp(&b.1))
            });
            orders.push(v.into_iter().map(|(_, _, id)| id).collect());
        }

        self.commit_order(&orders, &snap_ids);
    }

    pub fn locate_drop(
        &self,
        _x: f32,
        y: f32,
        exclude: &[u64],
    ) -> Option<(usize, Option<u64>)> {
        let rects = self.page_rects();
        let page = rects
            .iter()
            .find(|(_, top, h)| y >= *top && y < top + h)
            .map(|(p, _, _)| *p)?;

        let mut best: Option<(f32, u64)> = None;
        for b in self.blocks.iter().filter(|b| b.page == page) {
            if exclude.contains(&b.id) {
                continue;
            }
            let center = b.target_y + b.h / 2.0;
            if center >= y {
                continue;
            }
            if best.map_or(true, |(c, _)| center > c) {
                best = Some((center, b.id));
            }
        }
        Some((page, best.map(|(_, id)| id)))
    }

    pub fn step(&mut self, dt: f32) -> bool {
        let dt = dt.clamp(0.0, 0.032);
        let sub_count = if dt <= 0.0 {
            1
        } else {
            ((dt / crate::types::DT).ceil() as u32).max(1)
        };
        let sub_dt = if dt <= 0.0 {
            0.0
        } else {
            dt / sub_count as f32
        };
        for _ in 0..sub_count {
            self.substep(sub_dt);
        }
        self.blocks.iter().any(|b| b.phase != Phase::Idle)
    }

    fn substep(&mut self, dt: f32) {
        let mode = self.mode;
        let block_gap = self.config.block_gap;
        let snapshot = self.blocks.clone();

        for b in self.blocks.iter_mut() {
            if b.phase != Phase::Waiting {
                continue;
            }
            if mode == Mode::Ripple && b.delay > 0 {
                let step_delay_ms = (dt * 1000.0) as u32;
                b.delay = b.delay.saturating_sub(step_delay_ms);
            }
            if integrators::should_release(b, &snapshot, mode, block_gap) {
                b.phase = Phase::Moving;
                b.vel = if mode == Mode::Fall {
                    integrators::inherit_vel(b, &snapshot)
                } else {
                    0.0
                };
            }
        }

        if dt > 0.0 {
            for b in self.blocks.iter_mut() {
                if b.phase != Phase::Moving {
                    continue;
                }
                match mode {
                    Mode::Fall => integrators::integrate_fall(b, dt),
                    Mode::Spring | Mode::Ripple => integrators::integrate_spring(b, dt),
                }
            }
        }
    }

    pub fn page_rects(&self) -> Vec<(usize, f32, f32)> {
        layout::page_rects(&self.blocks, self.page_count, &self.config)
    }

    pub fn page_order(&self, page: usize) -> Vec<u64> {
        let mut v: Vec<(usize, f32, u64)> = self
            .blocks
            .iter()
            .enumerate()
            .filter(|(_, b)| b.page == page)
            .map(|(i, b)| (i, b.target_y, b.id))
            .collect();
        v.sort_by(|a, b| {
            a.1.partial_cmp(&b.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.0.cmp(&b.0))
        });
        v.into_iter().map(|(_, _, id)| id).collect()
    }

    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    pub fn block(&self, id: u64) -> Option<&Block> {
        self.blocks.iter().find(|b| b.id == id)
    }
}
