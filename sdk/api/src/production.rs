//! Immutable production declarations and physical configuration read points.
//! House counters, factory queues, installed upgrades and acquired plans stay
//! with their existing host owners. Resource queries never cache live answers.
use serde::{Deserialize, Serialize};

pub const BASE_PRODUCTION_GROUPS: [&str; 6] =
    ["POWER", "FACTORY", "BARRACKS", "RADAR", "TECH", "PROC"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductionObjectKind {
    Unit,
    Infantry,
    Building,
    Aircraft,
}

/// Original category and category-local array index, not a global entity ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionTypeRef {
    pub category: ProductionObjectKind,
    pub ordinal: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionRegisteredType {
    pub identity: ProductionTypeRef,
    pub name: String,
}

/// Prefix lengths at the reached reader. Later allocations cannot resolve an
/// earlier unknown reference. The ordered catalogs below are shared per pass.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionRegistryPrefix {
    pub buildings: u32,
    pub countries: u32,
    pub technos: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProductionReadSite {
    ProcessEntry,
    BeforeTypeData,
    TechnoPrerequisite {
        object: ProductionTypeRef,
    },
    TechnoExtension {
        object: ProductionTypeRef,
    },
    BuildingExtension {
        object: ProductionTypeRef,
        /// Recognized Type factory category at this exact read; unknown RTTIs
        /// are None. This fact does not decide any extension's skip policy.
        factory: Option<ProductionObjectKind>,
        unit_repair: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionReadPoint {
    pub prefix: ProductionRegistryPrefix,
    pub site: ProductionReadSite,
}

/// Read-only result of the host's rules-registry-view configuration service.
/// This view has no extension phase or interpreted gameplay fields. The guest
/// owns its own configuration phases, including calls while a module is off.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionRegistryView {
    pub buildings: Vec<String>,
    pub countries: Vec<String>,
    /// Native construction order across all four Techno families.
    pub technos: Vec<ProductionRegisteredType>,
    pub reads: Vec<ProductionReadPoint>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProductionResourceContext {
    Catalog,
    Object {
        category: ProductionObjectKind,
        name: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "index",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum ProductionRequirement {
    Building(u32),
    /// Full group index, including the six fixed initial groups.
    Group(u32),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionGroup {
    pub name: String,
    pub members: Vec<ProductionRequirement>,
    pub alternates: Vec<ProductionTypeRef>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductionPrerequisiteRule {
    #[default]
    Retail,
    /// Active direct buildings or matching installed upgrades; alternates
    /// read the existing owned counter. Members are never recursive.
    ActiveAndInstalled,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductionBuildLimitRule {
    #[default]
    Retail,
    /// Signed remaining capacity using tracked conversion partners/installed
    /// upgrades, production history for negative limits and held exceptions.
    TrackedCapacity,
}

/// Independent selectors for existing shared production owners. All-false and
/// Retail enums retain the stock route; there is no global compatibility mode.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ProductionSemantics {
    pub prerequisite_rule: ProductionPrerequisiteRule,
    pub build_limit_rule: ProductionBuildLimitRule,
    pub override_uses_groups: bool,
    pub require_factory_for_build: bool,
    pub ai_checks_build_limit: bool,
    pub negative_one_tech_level_human_only: bool,
    pub campaign_ignores_shell_superweapon_option: bool,
    pub factory_checks_limit_before_search: bool,
    pub factory_respects_explicit_sources: bool,
    pub factory_retains_offline_status: bool,
    pub deactivated_factory_is_offline: bool,
    pub factory_honors_stop_first: bool,
    pub factory_country_plans: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductionObjectPolicy {
    pub prerequisites: Vec<ProductionRequirement>,
    pub prerequisite_override: Vec<ProductionRequirement>,
    /// BuildingType ordinals; ordered lists retain duplicates.
    pub built_at: Vec<u32>,
    /// Country ordinals bound at the physical read point.
    pub factory_owners: Vec<u32>,
    pub forbidden_factory_owners: Vec<u32>,
    pub permanent_factory_plan: bool,
    pub factory_has_all_plans: bool,
    pub factory_explicit_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProductionResourcePolicy {
    Catalog {
        groups: Vec<ProductionGroup>,
        semantics: ProductionSemantics,
        /// Native difficulty slots, not UI display order.
        ai_bypass_build_limit: [bool; 3],
    },
    Object {
        policy: ProductionObjectPolicy,
    },
}

impl ProductionResourcePolicy {
    pub(crate) fn validate_for(&self, context: &ProductionResourceContext) -> Result<(), String> {
        match (self, context) {
            (Self::Catalog { groups, .. }, ProductionResourceContext::Catalog) => {
                if groups.len() < BASE_PRODUCTION_GROUPS.len()
                    || groups.len() > u32::MAX as usize
                    || groups.iter().zip(BASE_PRODUCTION_GROUPS).any(|(g, name)| g.name != name)
                    || groups.iter().any(|group| {
                        group.name.chars().count() > 31
                            || group.members.iter().any(|entry| {
                                matches!(entry, ProductionRequirement::Group(index) if *index as usize >= groups.len())
                            })
                    })
                {
                    return Err("invalid production group catalog".into());
                }
            }
            (Self::Object { policy }, ProductionResourceContext::Object { category, .. }) => {
                if *category != ProductionObjectKind::Building && policy.factory_explicit_only {
                    return Err("only buildings may be explicit source factories".into());
                }
            }
            _ => return Err("production policy does not match its resource query".into()),
        }
        // Type/Country ordinals and object Group references are checked against
        // the complete host catalogs before any resource binding is committed.
        Ok(())
    }
}
