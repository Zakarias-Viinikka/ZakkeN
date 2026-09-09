uniffi::setup_scaffolding!();

pub mod diff_logic;

#[cfg(target_arch = "wasm32")]
pub mod custom_text_area;

#[cfg(target_arch = "wasm32")]
pub mod helper;

#[cfg(target_arch = "wasm32")]
pub mod text_block;

pub mod when_to_diff;

pub mod when_to_diff_timer;
