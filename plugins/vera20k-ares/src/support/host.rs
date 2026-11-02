//! Narrow configuration and numeric imports. No guest file, clock, or network access.
use serde::{Serialize, de::DeserializeOwned};
use std::cell::RefCell;
use vera_mod_api::DamageKernelInput;

pub trait Host {
    fn config(&self, section: &str, key: &str) -> Option<String>;
    fn config_read(
        &self,
        section: &str,
        key: &str,
        reader: &str,
        current: &[u8],
    ) -> Result<Vec<u8>, String>;
    fn registry(&self, section: &str) -> Vec<String>;
    fn damage_kernel(&self, input: &DamageKernelInput) -> Result<i32, String>;
}

pub struct ConfigView<'a, H: Host> {
    host: &'a H,
    error: RefCell<Option<String>>,
}

impl<'a, H: Host> ConfigView<'a, H> {
    pub fn new(host: &'a H) -> Self {
        Self {
            host,
            error: RefCell::new(None),
        }
    }
    pub fn section(&self, name: &str) -> Option<Section<'_, H>> {
        (!self.host.registry(name).is_empty()).then(|| self.section_or_empty(name))
    }
    pub fn section_or_empty(&self, name: &str) -> Section<'_, H> {
        Section {
            view: self,
            name: name.to_owned(),
        }
    }
    pub fn finish(&self) -> Result<(), String> {
        self.error.borrow().clone().map_or(Ok(()), Err)
    }
}

pub struct Section<'a, H: Host> {
    view: &'a ConfigView<'a, H>,
    name: String,
}

impl<H: Host> Section<'_, H> {
    fn read<T: Clone + Serialize + DeserializeOwned>(
        &self,
        key: &str,
        reader: &str,
        current: T,
    ) -> T {
        let outcome = serde_json::to_vec(&current)
            .map_err(|e| e.to_string())
            .and_then(|bytes| self.view.host.config_read(&self.name, key, reader, &bytes))
            .and_then(|bytes| serde_json::from_slice(&bytes).map_err(|e| e.to_string()));
        match outcome {
            Ok(value) => value,
            Err(error) => {
                self.view
                    .error
                    .borrow_mut()
                    .get_or_insert_with(|| format!("[{}] {}: {}", self.name, key, error));
                current
            }
        }
    }
    pub fn physical_keys(&self) -> Vec<String> {
        self.view.host.registry(&self.name)
    }
    pub fn is_present(&self, key: &str) -> bool {
        self.view.host.config(&self.name, key).is_some()
    }
    pub fn read_int(&self, key: &str, current: i32) -> i32 {
        self.read(key, "int", current)
    }
    pub fn read_ares_int(&self, key: &str, current: i32) -> i32 {
        self.read(key, "ares-int", current)
    }
    pub fn read_bool(&self, key: &str, current: bool) -> bool {
        self.read(key, "bool", current)
    }
    pub fn read_double(&self, key: &str, current: f64) -> f64 {
        f64::from_bits(self.read(key, "double-bits", current.to_bits()))
    }
    pub fn read_phobos_nullable_int(&self, key: &str, current: Option<i32>) -> Option<i32> {
        self.read(key, "nullable-int", current)
    }
    pub fn read_phobos_nullable_bool(&self, key: &str, current: Option<bool>) -> Option<bool> {
        self.read(key, "nullable-bool", current)
    }
    pub fn read_phobos_nullable_double(&self, key: &str, current: Option<f64>) -> Option<f64> {
        self.read(key, "nullable-double-bits", current.map(f64::to_bits))
            .map(f64::from_bits)
    }
    pub fn read_string(&self, key: &str, current: &str, capacity: usize) -> String {
        self.read(key, &format!("string:{capacity}"), current.to_owned())
    }
    pub fn read_name(&self, key: &str, capacity: usize) -> Option<String> {
        self.read(key, &format!("name:{capacity}"), None::<String>)
    }
    pub fn read_list(&self, key: &str, capacity: usize) -> Option<Vec<String>> {
        self.read(key, &format!("list:{capacity}"), None::<Vec<String>>)
    }
    pub fn read_int_list(&self, key: &str) -> Option<Vec<i32>> {
        self.read(key, "int-list", None::<Vec<i32>>)
    }
    pub fn read_color_rgb(&self, key: &str, current: [u8; 3], _: Option<()>) -> [u8; 3] {
        self.read(key, "color-rgb", current)
    }
    pub fn read_phobos_nullable_with(
        &self,
        key: &str,
        current: Option<String>,
        _parse: impl Fn(&str) -> Option<String>,
    ) -> Option<String> {
        self.read(key, "nullable-name:128", current)
    }
}

#[cfg(target_arch = "wasm32")]
pub struct WasmHost;

#[cfg(target_arch = "wasm32")]
impl Host for WasmHost {
    fn config(&self, section: &str, key: &str) -> Option<String> {
        crate::bindings::vera::mod_sdk::host::config(section, key)
    }
    fn config_read(
        &self,
        section: &str,
        key: &str,
        reader: &str,
        current: &[u8],
    ) -> Result<Vec<u8>, String> {
        crate::bindings::vera::mod_sdk::host::config_read(section, key, reader, current)
    }
    fn registry(&self, section: &str) -> Vec<String> {
        crate::bindings::vera::mod_sdk::host::registry(section)
    }
    fn damage_kernel(&self, input: &DamageKernelInput) -> Result<i32, String> {
        let bytes = serde_json::to_vec(input).map_err(|e| e.to_string())?;
        let result = crate::bindings::vera::mod_sdk::host::damage_kernel(&bytes)?;
        serde_json::from_slice(&result).map_err(|e| e.to_string())
    }
}
