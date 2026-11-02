//! Process lifecycle boundaries delivered only to components that declare the
//! optional configure_boundary capability. Frozen saved games never replay them.
use serde::{Deserialize, Serialize};

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
