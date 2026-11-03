//! One frozen cameo catalog, queried at resource binding rather than per icon.
//! Source reads retain their physical Rules/ART chronology; decoding and atlas
//! resources remain host-owned.
use crate::{ImmutableIniSectionRef, ProductionTypeRef};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const MAX_CAMEO_FILENAME_BYTES: usize = 256;
pub const MAX_CAMEO_DECLARATIONS: usize = 65_536;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CameoReadPoint {
    Techno {
        object: ProductionTypeRef,
        art: ImmutableIniSectionRef,
    },
    SuperWeapon {
        ordinal: u32,
        section: String,
    },
}

/// A separate source domain: existing closed production and Foundation views
/// keep their wire shapes for older components.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CameoRegistryView {
    pub reads: Vec<CameoReadPoint>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CameoResourceContext {}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TechnoCameoDeclaration {
    pub object: ProductionTypeRef,
    pub normal: Option<String>,
    pub promoted: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuperWeaponCameoDeclaration {
    pub ordinal: u32,
    pub filename: String,
}

/// Some catalog selects its declarations, with stock art for omitted entries.
/// An empty catalog therefore explicitly selects stock even in a legacy host.
/// An absent outer policy preserves the host profile. The host validates every
/// identity before publishing this sparse catalog atomically.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CameoResourcePolicy {
    pub technos: Vec<TechnoCameoDeclaration>,
    pub super_weapons: Vec<SuperWeaponCameoDeclaration>,
}

impl CameoResourcePolicy {
    pub fn validate(&self) -> Result<(), String> {
        if self.technos.len().saturating_add(self.super_weapons.len()) > MAX_CAMEO_DECLARATIONS {
            return Err("cameo catalog exceeds its declaration limit".into());
        }
        let valid_name = |name: &str| {
            !name.is_empty()
                && name.len() <= MAX_CAMEO_FILENAME_BYTES
                && !name.chars().any(char::is_control)
        };
        let mut technos = BTreeSet::new();
        for entry in &self.technos {
            if !technos.insert(entry.object)
                || entry
                    .normal
                    .as_deref()
                    .is_some_and(|name| !valid_name(name))
                || entry
                    .promoted
                    .as_deref()
                    .is_some_and(|name| !valid_name(name))
            {
                return Err("invalid or duplicate Techno cameo declaration".into());
            }
        }
        let mut supers = BTreeSet::new();
        for entry in &self.super_weapons {
            if !supers.insert(entry.ordinal) || !valid_name(&entry.filename) {
                return Err("invalid or duplicate SuperWeapon cameo declaration".into());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{API_VERSION, Context, Plan, ProductionObjectKind, Request};

    #[test]
    fn cameo_catalogs_are_static_unique_and_separate_from_entity_presentation() {
        let request = Request {
            api_version: API_VERSION,
            context: Context::CameoResource(CameoResourceContext {}),
            config: vec![],
            state: vec![],
        };
        request.validate().unwrap();
        let mut plan = Plan::unchanged(&request);
        assert!(
            !serde_json::to_string(&plan)
                .unwrap()
                .contains("cameo_resource")
        );
        let mut catalog = CameoResourcePolicy::default();
        catalog.technos.push(TechnoCameoDeclaration {
            object: ProductionTypeRef {
                category: ProductionObjectKind::Unit,
                ordinal: 0,
            },
            normal: Some("none".into()),
            promoted: None,
        });
        catalog.super_weapons.push(SuperWeaponCameoDeclaration {
            ordinal: 0,
            filename: "power.pcx".into(),
        });
        plan.cameo_resource = Some(catalog.clone());
        plan.validate_for(&request).unwrap();
        let mut foreign = request.clone();
        foreign.context = Context::MatchRules(crate::MatchRulesContext {});
        assert!(plan.validate_for(&foreign).is_err());
        plan.state.push(1);
        assert!(plan.validate_for(&request).is_err());
        plan.state.clear();
        catalog.technos.push(catalog.technos[0].clone());
        assert!(catalog.validate().is_err());
        catalog.technos.pop();
        catalog.super_weapons[0].filename = "x".repeat(MAX_CAMEO_FILENAME_BYTES + 1);
        assert!(catalog.validate().is_err());
        plan.cameo_resource = Some(CameoResourcePolicy::default());
        plan.validate_for(&request).unwrap();
    }
}
