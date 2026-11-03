//! Decoded host replies exercise the real component path without duplicating
//! the native Find or nullable-int parser. Core tests own those readers.
use super::*;
use crate::component::tests::{FrozenHost, frozen_query};
use vera_mod_api::{
    ConfigurationBoundary, ConfigureBoundaryContext, ConfigureContext,
    ProductionObjectKind as Kind, ProductionReadPoint, ProductionReadSite,
    ProductionRegisteredType, ProductionRegistryPrefix, ProductionRegistryView,
    ProductionTypeRef as Ref, TypeConversionPolicy, TypeResourceContext, WEAPON_USER_INFANTRY,
    WEAPON_USER_UNIT,
};

fn identity(category: Kind, ordinal: u32) -> Ref {
    Ref { category, ordinal }
}

struct ConversionHost(u8);

impl ConversionHost {
    fn view(&self) -> ProductionRegistryView {
        let unit = identity(Kind::Unit, 0);
        let infantry = identity(Kind::Infantry, 0);
        let point = |object, technos| ProductionReadPoint {
            prefix: ProductionRegistryPrefix {
                buildings: u32::from(technos > 2),
                countries: 0,
                technos,
            },
            site: ProductionReadSite::TechnoExtension { object },
        };
        ProductionRegistryView {
            buildings: vec!["TARGET".into()],
            technos: [
                (unit, "SAME"),
                (infantry, "SAME"),
                (identity(Kind::Building, 0), "TARGET"),
                (identity(Kind::Unit, 1), "TARGET"),
            ]
            .into_iter()
            .map(|(identity, name)| ProductionRegisteredType {
                identity,
                name: name.into(),
            })
            .collect(),
            reads: match self.0 {
                0 => vec![point(unit, 2), point(infantry, 4)],
                1 | 3 | 4 => vec![point(unit, 4)],
                2 => vec![point(infantry, 4)],
                _ => vec![],
            },
            ..Default::default()
        }
    }
}

impl Host for ConversionHost {
    fn config(&self, section: &str, key: &str) -> Option<String> {
        (matches!(
            (self.0, section, key),
            (0..=4, "SAME", "Convert.Script" | "TurretROT")
                | (4 | 5, "VeraFeatures", "AresTypeConversion")
        ))
        .then(|| "decoded native reader reply".into())
    }

    fn config_read(
        &self,
        section: &str,
        key: &str,
        reader: &str,
        current: &[u8],
    ) -> Result<Vec<u8>, String> {
        if reader == "faction-registry-view" {
            // This domain fixture has no Side/Country reader dispatch.
            assert_eq!((section, key, current), ("", "", b"null".as_slice()));
            return serde_json::to_vec(&vera_mod_api::FactionRegistryView::default())
                .map_err(|e| e.to_string());
        }
        if reader == "rules-registry-view" {
            assert_ne!(self.0, 9, "final projection is not another physical pass");
            return serde_json::to_vec(&self.view()).map_err(|e| e.to_string());
        }
        match (section, key, reader) {
            ("VeraFeatures", "AresTypeConversion", "bool") => {
                serde_json::to_vec(&(self.0 == 5)).map_err(|e| e.to_string())
            }
            ("SAME", "TurretROT", "ares-nullable-int") => match self.0 {
                0 => Ok(b"-255".to_vec()),
                // Empty/invalid, including native `none`, do not clear an
                // already-set Ares nullable int as Phobos would.
                1 | 2 => Ok(current.to_vec()),
                3 => serde_json::to_vec(&i32::MIN).map_err(|e| e.to_string()),
                4 => serde_json::to_vec(&i32::MAX).map_err(|e| e.to_string()),
                _ => panic!("unexpected nullable-int visit"),
            },
            (
                "SAME",
                "Convert.Script",
                scope @ ("ares-ref:$technos@$vehicles" | "ares-ref:$technos@$infantry"),
            ) => {
                let source = if scope.ends_with("@$vehicles") {
                    identity(Kind::Unit, 0)
                } else {
                    identity(Kind::Infantry, 0)
                };
                let view = self.view();
                let prefix = view
                    .reads
                    .iter()
                    .find_map(|read| match read.site {
                        ProductionReadSite::TechnoExtension { object } if object == source => {
                            Some(read.prefix.technos)
                        }
                        _ => None,
                    })
                    .expect("reader must be scoped to an actual common suffix");
                let decoded = match (self.0, source.category) {
                    (0, Kind::Unit) => {
                        assert_eq!(prefix, 2);
                        None // TARGET is allocated only after this read.
                    }
                    (0, Kind::Infantry) | (1, Kind::Unit) => {
                        assert_eq!(prefix, 4);
                        // Common construction order finds Building TARGET
                        // first, even for a Unit/Infantry source.
                        Some(identity(Kind::Building, 0))
                    }
                    (2, Kind::Infantry) => None, // explicit native none
                    (3, Kind::Unit) => return Ok(current.to_vec()), // unknown
                    (4, Kind::Unit) => Some(identity(Kind::Unit, 1)),
                    _ => panic!("unexpected scoped Find visit"),
                };
                serde_json::to_vec(&decoded).map_err(|e| e.to_string())
            }
            _ => panic!("unexpected conversion reader {section}/{key}/{reader}"),
        }
    }

    fn registry(&self, registry: &str) -> Vec<String> {
        match (self.0, registry) {
            (0 | 1 | 3 | 4, "$vehicles") | (0 | 2, "$infantry") => vec!["SAME".into()],
            _ => vec![],
        }
    }

    fn damage_kernel(&self, _: &vera_mod_api::DamageKernelInput) -> Result<i32, String> {
        panic!("conversion configuration has no damage kernel")
    }
}

fn configure(config: Vec<u8>, pass: u8) -> Vec<u8> {
    let request = Request {
        api_version: API_VERSION,
        context: Context::Configure(ConfigureContext {
            pass: pass.into(),
            source: "decoded-conversion-fixture".into(),
            final_pass: pass == 9,
            object_types: vec!["SAME".into(), "TARGET".into()],
            warheads: vec![],
        }),
        config,
        state: vec![],
    };
    let plan = evaluate(&request, &ConversionHost(pass)).unwrap();
    plan.validate_for(&request).unwrap();
    plan.config.unwrap()
}

fn policy(config: &[u8], user: u8) -> TypeConversionPolicy {
    frozen_query(
        config,
        Context::TypeResource(TypeResourceContext {
            type_name: "SAME".into(),
            user,
        }),
    )
    .type_resource
    .unwrap()
    .conversion
    .unwrap()
}

fn handler(config: &[u8]) -> Option<bool> {
    frozen_query(
        config,
        Context::MatchRules(vera_mod_api::MatchRulesContext {}),
    )
    .match_rules
    .unwrap()
    .script_type_conversion
}

fn boundary(config: Vec<u8>, boundary: ConfigurationBoundary) -> Vec<u8> {
    let request = Request {
        api_version: API_VERSION,
        context: Context::ConfigureBoundary(ConfigureBoundaryContext { boundary }),
        config,
        state: vec![],
    };
    let plan = evaluate(&request, &FrozenHost).unwrap();
    plan.validate_for(&request).unwrap();
    plan.config.unwrap()
}

#[test]
fn conversion_freezes_scoped_identity_and_full_signed_nullable_rotation() {
    let first = configure(vec![], 0);
    assert_eq!(policy(&first, WEAPON_USER_UNIT).script_target, None);
    assert_eq!(
        policy(&first, WEAPON_USER_INFANTRY).script_target,
        Some(identity(Kind::Building, 0))
    );
    assert_eq!(policy(&first, WEAPON_USER_UNIT).secondary_rot, Some(-255));
    let later = configure(first, 1);
    assert_eq!(
        policy(&later, WEAPON_USER_UNIT).script_target,
        Some(identity(Kind::Building, 0))
    );
    assert_eq!(policy(&later, WEAPON_USER_UNIT).secondary_rot, Some(-255));
    let cleared = configure(later, 2);
    assert_eq!(
        policy(&cleared, WEAPON_USER_INFANTRY),
        TypeConversionPolicy {
            script_target: None,
            secondary_rot: Some(-255)
        }
    );
    let frozen = configure(configure(cleared, 3), 9);
    assert_eq!(
        policy(&frozen, WEAPON_USER_UNIT),
        TypeConversionPolicy {
            script_target: Some(identity(Kind::Building, 0)),
            secondary_rot: Some(i32::MIN),
        }
    );
    assert_eq!(handler(&frozen), Some(true));
}

#[test]
fn conversion_accumulates_while_off_and_boundaries_restore_fresh_handler() {
    let disabled = configure(vec![], 4);
    assert_eq!(handler(&disabled), Some(false));
    assert_eq!(
        policy(&disabled, WEAPON_USER_UNIT),
        TypeConversionPolicy::default()
    );
    let enabled = configure(configure(disabled, 5), 9);
    assert_eq!(
        policy(&enabled, WEAPON_USER_UNIT),
        TypeConversionPolicy {
            script_target: Some(identity(Kind::Unit, 1)),
            secondary_rot: Some(i32::MAX),
        }
    );
    let reset = boundary(enabled.clone(), ConfigurationBoundary::TypeReset);
    assert_eq!(
        policy(&reset, WEAPON_USER_UNIT),
        TypeConversionPolicy::default()
    );
    assert_eq!(
        handler(&reset),
        Some(true),
        "handler availability does not depend on nonempty type policies"
    );
    let cleared = boundary(enabled, ConfigurationBoundary::ScenarioClear);
    assert_eq!(
        policy(&cleared, WEAPON_USER_UNIT),
        TypeConversionPolicy::default()
    );
    let json: serde_json::Value = serde_json::from_slice(&cleared).unwrap();
    assert_eq!(json["type_conversion"]["types"], serde_json::json!({}));
}
