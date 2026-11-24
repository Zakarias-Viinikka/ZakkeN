#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Idle,
    Waiting,
    Moving,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, uniffi::Enum)]
pub enum Mode {
    Fall,
    Spring,
    Ripple,
}

#[derive(Clone, Copy, Debug, uniffi::Record)]
pub struct Config {
    pub block_gap: f32,
    pub top_pad: f32,
    pub edge_pad: f32,
    pub page_gap: f32,
    pub min_page_h: f32,
    pub block_w: f32,
    pub mode: Mode,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub id: u64,
    pub page: usize,
    pub y: f32,
    pub target_y: f32,
    pub h: f32,
    pub vel: f32,
    pub phase: Phase,
    pub prev: Option<u64>,
    pub delay: u32,
}

pub const ACCEL: f32 = 1500.0;
pub const MAX_V: f32 = 600.0;
pub const DT: f32 = 0.016;
pub const CONTACT: f32 = 3.0;
pub const SPRING_K: f32 = 380.0;
pub const SPRING_C: f32 = 22.0;
pub const SPRING_SETTLE_POS: f32 = 0.2;
pub const SPRING_SETTLE_VEL: f32 = 5.0;
pub const SPRING_MAX_V: f32 = 1200.0;
pub const RIPPLE_STAGGER_MS: u32 = 28;
