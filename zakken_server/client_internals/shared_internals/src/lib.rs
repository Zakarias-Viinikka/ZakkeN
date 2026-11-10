mod conn_status;
mod drift_version;
mod request_builder;
mod router;

pub use conn_status::ConnStatus;
pub use drift_version::{get_drift_version, set_drift_version};
pub use request_builder::{ClientCommand, build_request};
pub use router::{Router, RouterHandle};

#[cfg(target_arch = "wasm32")]
mod socket_web;

#[cfg(target_os = "android")]
mod socket_android;