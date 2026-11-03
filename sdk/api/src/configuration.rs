//! Process lifecycle boundaries delivered only to components that declare the
//! optional configure_boundary capability. Frozen saved games never replay them.
use serde::{Deserialize, Serialize};

/// A startup-selected immutable INI, never a guest-provided filesystem path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FixedIniSource {
    FixedArt,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImmutableIniSectionRef {
    pub source: FixedIniSource,
    pub source_hash: u64,
    pub section: String,
}

impl ImmutableIniSectionRef {
    pub fn validate(&self) -> Result<(), String> {
        if self.section.is_empty()
            || self.section.len() > 256
            || self.section.chars().any(char::is_control)
        {
            return Err("invalid immutable INI section name".into());
        }
        Ok(())
    }
}

/// Input for the shared fixed-source scalar readers. Large source contents
/// stay in the host; only the selected scalar and its default cross the ABI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImmutableIniValue<T> {
    pub reference: ImmutableIniSectionRef,
    pub current: T,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigurationBoundary {
    /// A new ordinary scenario recreates global and Type configuration.
    ScenarioClear,
    /// Native Type destruction retains process-global configuration (for
    /// example generated-map admission), but discards Type-bound declarations.
    TypeReset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigureBoundaryContext {
    pub boundary: ConfigurationBoundary,
}
