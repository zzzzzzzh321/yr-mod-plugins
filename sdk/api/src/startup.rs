//! Pure resource recipes selected before Rules configuration. The host keeps
//! file resolution, binary/INI parsers and the sole resource catalogs.
use serde::{Deserialize, Serialize};

pub const MAX_STARTUP_RESOURCES: usize = 128;

/// Pure startup admission for the two existing external-launch identity gates.
/// Country allocation, campaign record selection, peers and colors keep their
/// original owners. An absent policy retains the host profile; explicit false
/// selects the retail limit even in a legacy host.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpawnIdentityPolicy {
    pub extended_countries: bool,
    pub campaign_side_byte: bool,
}

/// A separate optional capability, not another StartupResourceContext stage:
/// existing guests with the three-stage hook cannot decode new stages.
/// The host creates the fixed speech catalog before Rules configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaCatalogPolicy {
    Retail,
    RegisteredVoiceColumns,
}

/// Select only the existing House Allies and Techno Prerequisite /
/// PrerequisiteOverride ReadString buffers before cold Rules processing.
/// General prerequisite groups and physical INI line admission are unchanged.
/// A missing declaration preserves the host's existing reader profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LongListReadersPolicy {
    #[serde(rename = "retail_128")]
    Retail128,
    #[serde(rename = "extended_2048")]
    Extended2048,
}

/// Select the host's fixed master-list readers before each physical Rules pass.
/// The host owns section names, parsing, registration and dispatch order; this
/// declaration cannot add keys or inspect gameplay configuration or state.
/// A missing declaration preserves the host's reader profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MasterTypeListsPolicy {
    pub weapon_types: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "stage", rename_all = "snake_case", deny_unknown_fields)]
pub enum StartupResourceContext {
    SourceComposition,
    AudioCatalog,
    StringSupplements { base_language: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncludeSemantics {
    Retail,
    HashIncludeOnce,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StartupAudioPair {
    pub index: String,
    pub bag: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "semantics", rename_all = "snake_case", deny_unknown_fields)]
pub enum StartupAudioCatalog {
    Retail,
    OrderedPairs { pairs: Vec<StartupAudioPair> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CsfLanguageGate {
    Ignore,
    MatchBaseOrNeutral,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StartupCsfCandidate {
    pub name: String,
    pub language: CsfLanguageGate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "semantics", rename_all = "snake_case", deny_unknown_fields)]
pub enum StartupCsfComposition {
    BaseOnly,
    OrderedRecords {
        candidates: Vec<StartupCsfCandidate>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "stage", rename_all = "snake_case", deny_unknown_fields)]
pub enum StartupResourcePolicy {
    SourceComposition { include: IncludeSemantics },
    AudioCatalog { catalog: StartupAudioCatalog },
    StringSupplements { composition: StartupCsfComposition },
}

fn validate_name(name: &str) -> Result<(), String> {
    // Structural wire bound only. The resource adapter must also validate
    // names through its existing logical VFS path admission before lookup.
    if name.is_empty() || name.len() > 256 || name.chars().any(char::is_control) {
        return Err("invalid startup resource name".into());
    }
    Ok(())
}

impl StartupResourcePolicy {
    pub fn validate_for(&self, context: &StartupResourceContext) -> Result<(), String> {
        match (self, context) {
            (Self::SourceComposition { .. }, StartupResourceContext::SourceComposition) => Ok(()),
            (Self::AudioCatalog { catalog }, StartupResourceContext::AudioCatalog) => {
                if let StartupAudioCatalog::OrderedPairs { pairs } = catalog {
                    if pairs.len() > MAX_STARTUP_RESOURCES {
                        return Err("too many startup audio pairs".into());
                    }
                    for pair in pairs {
                        validate_name(&pair.index)?;
                        validate_name(&pair.bag)?;
                    }
                }
                Ok(())
            }
            (
                Self::StringSupplements { composition },
                StartupResourceContext::StringSupplements { .. },
            ) => {
                if let StartupCsfComposition::OrderedRecords { candidates } = composition {
                    if candidates.len() > MAX_STARTUP_RESOURCES {
                        return Err("too many startup string supplements".into());
                    }
                    for candidate in candidates {
                        validate_name(&candidate.name)?;
                    }
                }
                Ok(())
            }
            _ => Err("startup resource policy does not match its query stage".into()),
        }
    }
}
