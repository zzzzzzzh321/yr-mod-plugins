//! VERA20K external gameplay reference. Features share one component and Host.
//!
//! Moved from VERA20K's Rust `sim/shield.rs` and `rules/phobos_shields.rs`.
//! Native references remain beside their corresponding behavior. No Ares or
//! Phobos C++ source or proprietary assets are incorporated.
mod component;
mod features;
mod support;

pub use component::manifest;

#[cfg(target_arch = "wasm32")]
mod bindings {
    wit_bindgen::generate!({ path: "../../sdk/wit", world: "gameplay" });
}

#[cfg(target_arch = "wasm32")]
struct Component;

#[cfg(target_arch = "wasm32")]
impl bindings::Guest for Component {
    fn evaluate(kind: String, input: Vec<u8>) -> Result<Vec<u8>, String> {
        if kind == "manifest" {
            return serde_json::to_vec(&manifest()).map_err(|e| e.to_string());
        }
        let request: vera_mod_api::Request =
            serde_json::from_slice(&input).map_err(|e| e.to_string())?;
        if kind != request.context.kind().as_str() {
            return Err("hook/context mismatch".into());
        }
        let plan = component::evaluate(&request, &support::WasmHost)?;
        serde_json::to_vec(&plan).map_err(|e| e.to_string())
    }
}

#[cfg(target_arch = "wasm32")]
bindings::export!(Component with_types_in bindings);
