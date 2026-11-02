//! Feature-independent guest access to the SDK host and stable numeric encoding.
pub(crate) mod codec;
mod host;
#[cfg(target_arch = "wasm32")]
pub(crate) use host::WasmHost;
pub(crate) use host::{ConfigView, Host, Section};
