//! Plumbing only. Every file under here is a "shared owner" — one file
//! per thing that is created in one place and touched by many others.
//! The file's header documents every toucher. See coding_style.md.

pub mod drag_in_progress_feedback;
pub mod visual_row_order;