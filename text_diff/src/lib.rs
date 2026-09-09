uniffi::setup_scaffolding!();

pub mod diff_logic;

#[cfg(target_arch = "wasm32")]
pub mod custom_text_area;

#[cfg(target_arch = "wasm32")]
pub mod helper;

#[cfg(target_arch = "wasm32")]
pub mod text_block;

pub mod text_diff_batching;
