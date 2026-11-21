#[derive(Clone, Copy)]
pub struct DragState {
    pub id: usize,
    pub offset: (f32, f32),
    pub current: (f32, f32),
    pub moved: bool,
    pub start: (f32, f32),
}

#[derive(Clone, Copy)]
pub struct RectDragState {
    pub offset: (f32, f32),
    pub current: (f32, f32),
    pub moved: bool,
    pub start: (f32, f32),
}

pub const DRAG_THRESHOLD: f32 = 4.0;
