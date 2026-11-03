//! Narrow configuration and numeric imports. No guest file, clock, or network access.
use serde::{Serialize, de::DeserializeOwned};
use std::cell::RefCell;
use vera_mod_api::{
    ArmorVersePolicy, DamageKernelInput, FactionLookup, FactionReadValue, FactionRegistryView,
    FoundationRegistryView, ImmutableIniSectionRef, ImmutableIniValue, NullableResourceName,
    ProductionRegistryView, ProductionTypeRef,
};

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
            registry: None,
        }
    }
    pub fn section_in_registry(&self, name: &str, registry: &str) -> Section<'_, H> {
        Section {
            view: self,
            name: name.to_owned(),
            registry: Some(registry.to_owned()),
        }
    }
    /// Canonical type identities at this physical Rules pass. These registries
    /// include types allocated by references, not just an INI list's keys.
    pub fn registered_names(&self, registry: &str) -> Vec<String> {
        self.host.registry(registry)
    }
    /// Shared physical read points and native identities, without interpreted
    /// gameplay fields. A missing/invalid view rejects configuration.
    pub fn rules_registry_view(&self) -> Result<ProductionRegistryView, String> {
        let bytes = self
            .host
            .config_read("", "", "rules-registry-view", b"null")?;
        serde_json::from_slice(&bytes).map_err(|e| format!("rules registry view: {e}"))
    }
    /// Actual Side/Country calls in this physical pass. No Type catalog or
    /// interpreted extension state is copied into the guest.
    pub fn faction_registry_view(&self) -> Result<FactionRegistryView, String> {
        let bytes = self
            .host
            .config_read("", "", "faction-registry-view", b"null")?;
        let view: FactionRegistryView =
            serde_json::from_slice(&bytes).map_err(|e| format!("faction registry view: {e}"))?;
        view.validate()?;
        Ok(view)
    }
    /// A scoped canonical native reader. All supported readers retain current
    /// for an absent physical key; present-empty still reaches the host.
    pub fn faction_read<T: Clone + Serialize + DeserializeOwned>(
        &self,
        read: u32,
        owner: &str,
        key: &str,
        reader: &str,
        current: T,
    ) -> T {
        if self.host.config(owner, key).is_none() {
            return current;
        }
        let input = FactionReadValue {
            read,
            current: current.clone(),
        };
        let outcome = serde_json::to_vec(&input)
            .map_err(|e| e.to_string())
            .and_then(|bytes| self.host.config_read(owner, key, reader, &bytes))
            .and_then(|bytes| serde_json::from_slice(&bytes).map_err(|e| e.to_string()));
        match outcome {
            Ok(value) => value,
            Err(error) => {
                self.error
                    .borrow_mut()
                    .get_or_insert_with(|| format!("faction read {read} [{owner}] {key}: {error}"));
                current
            }
        }
    }
    /// Literal lookup at this same read point, including constructor defaults.
    /// The native registry owns Find order and sentinel handling.
    pub fn faction_find<T: DeserializeOwned>(
        &self,
        read: u32,
        name: &str,
        family: &str,
    ) -> Option<T> {
        let input = FactionLookup {
            read,
            name: name.to_owned(),
        };
        let outcome = serde_json::to_vec(&input)
            .map_err(|e| e.to_string())
            .and_then(|bytes| {
                self.host
                    .config_read("", "", &format!("faction-find:{family}"), &bytes)
            })
            .and_then(|bytes| serde_json::from_slice(&bytes).map_err(|e| e.to_string()));
        match outcome {
            Ok(value) => value,
            Err(error) => {
                self.error.borrow_mut().get_or_insert_with(|| {
                    format!("faction literal {read} {family} {name}: {error}")
                });
                None
            }
        }
    }
    /// An optional fixed-source configuration domain. Old decoded hosts may
    /// omit it; once discovered, service or decoding errors must propagate.
    pub fn fixed_art_view(&self) -> Result<FoundationRegistryView, String> {
        if self.host.registry("$fixed-art").is_empty() {
            return Ok(FoundationRegistryView::default());
        }
        let bytes = self.host.config_read("", "", "fixed-art-view", b"null")?;
        serde_json::from_slice(&bytes).map_err(|e| format!("fixed ART view: {e}"))
    }
    fn read_source<T: Clone + Serialize + DeserializeOwned>(
        &self,
        reference: &ImmutableIniSectionRef,
        owner: &str,
        key: &str,
        reader: &str,
        current: T,
    ) -> T {
        // Rules-only presence probes cannot describe this source. Always ask
        // the canonical fixed-source reader, including absent/empty keys.
        let input = ImmutableIniValue {
            reference: reference.clone(),
            current: current.clone(),
        };
        let outcome = serde_json::to_vec(&input)
            .map_err(|e| e.to_string())
            .and_then(|bytes| self.host.config_read(owner, key, reader, &bytes))
            .and_then(|bytes| serde_json::from_slice(&bytes).map_err(|e| e.to_string()));
        match outcome {
            Ok(value) => value,
            Err(error) => {
                self.error.borrow_mut().get_or_insert_with(|| {
                    format!(
                        "fixed source [{}] {key} ({owner}): {error}",
                        reference.section
                    )
                });
                current
            }
        }
    }
    pub fn read_source_string(
        &self,
        reference: &ImmutableIniSectionRef,
        key: &str,
        current: &str,
        capacity: usize,
    ) -> String {
        self.read_source(
            reference,
            &reference.section,
            key,
            &format!("source-string:{capacity}"),
            current.to_owned(),
        )
    }
    pub fn read_stock_foundation(
        &self,
        reference: &ImmutableIniSectionRef,
        type_name: &str,
        current: u8,
    ) -> u8 {
        self.read_source(
            reference,
            type_name,
            "Foundation",
            "foundation-stock",
            current,
        )
    }
    /// Decode one already-tokenized verse through the host's canonical Ares
    /// parser100523E0. No section lookup or guest numeric parser is involved.
    pub fn parse_verse(&self, token: &str) -> ArmorVersePolicy {
        let outcome = serde_json::to_vec(token)
            .map_err(|e| e.to_string())
            .and_then(|bytes| self.host.config_read("", "", "verse-token", &bytes))
            .and_then(|bytes| serde_json::from_slice(&bytes).map_err(|e| e.to_string()));
        match outcome {
            Ok(value) => value,
            Err(error) => {
                self.error
                    .borrow_mut()
                    .get_or_insert_with(|| format!("armor verse token: {error}"));
                ArmorVersePolicy::default()
            }
        }
    }
    pub fn finish(&self) -> Result<(), String> {
        self.error.borrow().clone().map_or(Ok(()), Err)
    }
}

pub struct Section<'a, H: Host> {
    view: &'a ConfigView<'a, H>,
    name: String,
    registry: Option<String>,
}

impl<H: Host> Section<'_, H> {
    fn read<T: Clone + Serialize + DeserializeOwned>(
        &self,
        key: &str,
        reader: &str,
        current: T,
    ) -> T {
        // Native typed readers retain current for an absent key. Probe through
        // the host's own lookup (including projected Rules passes), avoiding
        // JSON/WIT round trips for thousands of unused extension defaults.
        // ReadString also trims/truncates its default, so always delegate it.
        // Present empty/invalid values still go through the native reader.
        if !reader.starts_with("string:") && !self.is_present(key) {
            return current;
        }
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
    pub fn read_ares_nullable_int(&self, key: &str, current: Option<i32>) -> Option<i32> {
        self.read(key, "ares-nullable-int", current)
    }
    pub fn read_ares_emp_threshold(&self, key: &str, current: i32) -> i32 {
        self.read(key, "ares-emp-threshold", current)
    }
    pub fn read_ares_nullable_mission(&self, key: &str, current: Option<u16>) -> Option<u16> {
        self.read(key, "ares-nullable-mission", current)
    }
    pub fn read_bool(&self, key: &str, current: bool) -> bool {
        self.read(key, "bool", current)
    }
    pub fn read_ares_bool(&self, key: &str, current: bool) -> bool {
        self.read(key, "ares-bool", current)
    }
    pub fn read_ares_nullable_bool(&self, key: &str, current: Option<bool>) -> Option<bool> {
        self.read(key, "ares-nullable-bool", current)
    }
    pub fn read_double(&self, key: &str, current: f64) -> f64 {
        f64::from_bits(self.read(key, "double-bits", current.to_bits()))
    }
    pub fn read_ares_double_bits(&self, key: &str, current: u64) -> u64 {
        self.read(key, "ares-double-bits", current)
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
    pub fn read_ares_registered_name(
        &self,
        key: &str,
        current: Option<String>,
        registry: &str,
    ) -> Option<String> {
        let reader = match &self.registry {
            Some(scope) => format!("ares-ref:{registry}@{scope}"),
            None => format!("ares-ref:{registry}"),
        };
        self.read(key, &reader, current)
    }
    /// Find a typed identity in the reached common Techno construction prefix;
    /// the host owns sentinel rejection, order, and unknown-value retention.
    pub fn read_ares_registered_type(
        &self,
        key: &str,
        current: Option<ProductionTypeRef>,
    ) -> Option<ProductionTypeRef> {
        let reader = match &self.registry {
            Some(scope) => format!("ares-ref:$technos@{scope}"),
            None => "ares-ref:$technos".into(),
        };
        self.read(key, &reader, current)
    }
    pub fn read_ares_nullable_registered_name(
        &self,
        key: &str,
        current: NullableResourceName,
        registry: &str,
    ) -> NullableResourceName {
        let reader = match &self.registry {
            Some(scope) => format!("ares-nullable-ref:{registry}@{scope}"),
            None => format!("ares-nullable-ref:{registry}"),
        };
        self.read(key, &reader, current)
    }
    pub fn read_list(&self, key: &str, capacity: usize) -> Option<Vec<String>> {
        self.read(key, &format!("list:{capacity}"), None::<Vec<String>>)
    }
    /// The canonical comma-only list reader preserves spaces inside each
    /// token; callers choose it separately from the whitespace-tokenizing list.
    pub fn read_comma_list(&self, key: &str, capacity: usize) -> Option<Vec<String>> {
        self.read(key, &format!("comma-list:{capacity}"), None::<Vec<String>>)
    }
    pub fn read_bool_prefix_reverse3(&self, key: &str, current: [bool; 3]) -> [bool; 3] {
        self.read(key, "bool-prefix-reverse3", current)
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

#[cfg(test)]
mod tests {
    use super::*;

    struct PresenceHost;
    impl Host for PresenceHost {
        fn config(&self, _: &str, key: &str) -> Option<String> {
            (key == "Empty").then(String::new)
        }
        fn config_read(
            &self,
            _: &str,
            key: &str,
            reader: &str,
            _: &[u8],
        ) -> Result<Vec<u8>, String> {
            match (key, reader) {
                ("Missing", "string:4") => Ok(br#""abc""#.to_vec()),
                ("Empty", "ares-int") => Ok(b"7".to_vec()),
                ("Empty", "nullable-int") => Err("reader failure".into()),
                ("", "verse-token") => Err("verse reader failure".into()),
                _ => panic!("absent values must retain their defaults without JSON round trips"),
            }
        }
        fn registry(&self, _: &str) -> Vec<String> {
            panic!("no registry needed")
        }
        fn damage_kernel(&self, _: &DamageKernelInput) -> Result<i32, String> {
            panic!("no damage calculation")
        }
    }

    #[test]
    fn absent_fields_retain_defaults_but_strings_still_use_native_copy() {
        let view = ConfigView::new(&PresenceHost);
        let section = view.section_or_empty("Unit");
        assert_eq!(section.read_ares_int("Missing", 3), 3);
        assert!(section.read_bool("Missing", true));
        assert_eq!(
            section.read_phobos_nullable_int("Missing", Some(9)),
            Some(9)
        );
        let special = f64::from_bits(0x7ff8_0000_0000_1234);
        assert_eq!(
            section.read_double("Missing", special).to_bits(),
            special.to_bits()
        );
        assert_eq!(section.read_list("Missing", 128), None);
        // Native ReadString trims/truncates even an absent key's default.
        assert_eq!(section.read_string("Missing", "abcdef", 4), "abc");
        view.finish().unwrap();
    }

    #[test]
    fn present_empty_fields_and_host_errors_still_reach_native_reader() {
        let view = ConfigView::new(&PresenceHost);
        let section = view.section_or_empty("Unit");
        assert_eq!(section.read_ares_int("Empty", 3), 7);
        assert_eq!(section.read_phobos_nullable_int("Empty", Some(9)), Some(9));
        assert!(view.finish().unwrap_err().contains("reader failure"));
    }

    #[test]
    fn verse_service_errors_invalidate_the_configuration() {
        let view = ConfigView::new(&PresenceHost);
        assert_eq!(
            view.parse_verse("opaque token"),
            ArmorVersePolicy::default()
        );
        assert!(view.finish().unwrap_err().contains("verse reader failure"));
    }
}
