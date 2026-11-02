//! Decoded native reader replies exercise physical visits and frozen routing;
//! the host tests own whitespace, integer and bounded string parsing.
use super::*;
use crate::component::tests::frozen_query;
use vera_mod_api::{
    ProductionObjectKind as Kind, ProductionReadPoint, ProductionReadSite,
    ProductionRegisteredType, ProductionRegistryView, ProductionTypeRef, SpotlightAttachment,
    SpotlightPolicy, TypeResourceContext, WEAPON_USER_BUILDING, WEAPON_USER_INFANTRY,
    WEAPON_USER_UNIT,
};

struct SpotlightHost(u8);

impl SpotlightHost {
    fn present(&self, section: &str, key: &str) -> bool {
        matches!(
            (self.0, section, key),
            (0..=2, "VeraFeatures", "AresSpotlight")
                | (0..=2, "SAME" | "STOCKONLY", "HasSpotlight")
                | (3..=6, "SAME", "Spotlight.AttachedTo")
                | (3, "SAME", "Spotlight.StartHeight" | "Spotlight.Distance")
                | (4, "SAME", "Spotlight.Distance")
                | (3, "SAME", "Spotlight.DisableRed" | "Spotlight.DisableBlue")
                | (
                    4,
                    "SAME",
                    "Spotlight.DisableRed" | "Spotlight.DisableGreen" | "Spotlight.DisableColor"
                )
        )
    }
}

impl Host for SpotlightHost {
    fn config(&self, section: &str, key: &str) -> Option<String> {
        self.present(section, key)
            .then(|| "decoded reader fixture".into())
    }

    fn config_read(
        &self,
        section: &str,
        key: &str,
        reader: &str,
        current: &[u8],
    ) -> Result<Vec<u8>, String> {
        if reader == "rules-registry-view" {
            let building = |ordinal| ProductionTypeRef {
                category: Kind::Building,
                ordinal,
            };
            let view = ProductionRegistryView {
                buildings: vec!["SAME".into(), "STOCKONLY".into()],
                technos: vec![
                    ProductionRegisteredType {
                        identity: building(0),
                        name: "SAME".into(),
                    },
                    ProductionRegisteredType {
                        identity: building(1),
                        name: "STOCKONLY".into(),
                    },
                ],
                reads: if self.0 <= 2 {
                    [building(0), building(1)]
                        .into_iter()
                        .map(|object| ProductionReadPoint {
                            prefix: vera_mod_api::ProductionRegistryPrefix {
                                buildings: 2,
                                countries: 0,
                                technos: 2,
                            },
                            site: ProductionReadSite::BuildingExtension {
                                object,
                                factory: None,
                                unit_repair: false,
                            },
                        })
                        .collect()
                } else {
                    vec![]
                },
                ..Default::default()
            };
            return serde_json::to_vec(&view).map_err(|e| e.to_string());
        }
        if !self.present(section, key) {
            return Ok(current.to_vec());
        }
        let value = match (self.0, section, key, reader) {
            (0..=2, "VeraFeatures", "AresSpotlight", "bool") => {
                serde_json::json!(self.0 == 1)
            }
            // The canonical readers' already-decoded replies for " yes":
            // Ares trims and accepts; retail keeps its separate prior value.
            (0, "SAME", "HasSpotlight", "ares-bool") => serde_json::json!(true),
            (0, "SAME", "HasSpotlight", "bool")
            | (1 | 2, _, "HasSpotlight", "bool" | "ares-bool") => {
                return Ok(current.to_vec());
            }
            (0, "STOCKONLY", "HasSpotlight", "bool") => serde_json::json!(true),
            (3, "SAME", "Spotlight.StartHeight", "ares-int") => serde_json::json!(-20),
            (3, "SAME", "Spotlight.Distance", "ares-int") => serde_json::json!(i32::MAX),
            (4, "SAME", "Spotlight.Distance", "ares-int") => return Ok(current.to_vec()),
            (3, "SAME", "Spotlight.AttachedTo", "string:2048") => {
                serde_json::json!("TuRrEt")
            }
            (4, "SAME", "Spotlight.AttachedTo", "string:2048") => {
                serde_json::json!("BARREL")
            }
            (5, "SAME", "Spotlight.AttachedTo", "string:2048") => {
                serde_json::json!("unknown attachment")
            }
            (6, "SAME", "Spotlight.AttachedTo", "string:2048") => serde_json::json!(""),
            (4, "SAME", "Spotlight.DisableRed", "ares-bool") => serde_json::json!(false),
            (_, "SAME", _, "ares-bool") => serde_json::json!(true),
            _ => panic!("unexpected spotlight reader {section}/{key}/{reader}"),
        };
        serde_json::to_vec(&value).map_err(|e| e.to_string())
    }

    fn registry(&self, registry: &str) -> Vec<String> {
        if registry == "$vehicles" && self.0 <= 6 {
            vec!["SAME".into(), "DEFAULT".into()]
        } else {
            vec![]
        }
    }

    fn damage_kernel(&self, _: &vera_mod_api::DamageKernelInput) -> Result<i32, String> {
        panic!("spotlight configuration has no damage kernel")
    }
}

fn configure(config: Vec<u8>, pass: u8) -> Vec<u8> {
    let request = Request {
        api_version: API_VERSION,
        context: Context::Configure(vera_mod_api::ConfigureContext {
            pass: pass.into(),
            source: "decoded-spotlight-fixture".into(),
            final_pass: pass == 9,
            object_types: vec!["SAME".into(), "STOCKONLY".into(), "DEFAULT".into()],
            warheads: vec![],
        }),
        config,
        state: vec![],
    };
    let plan = evaluate(&request, &SpotlightHost(pass)).unwrap();
    plan.validate_for(&request).unwrap();
    plan.config.unwrap()
}

fn spotlight(config: Vec<u8>, name: &str, user: u8) -> SpotlightPolicy {
    frozen_query(
        &config,
        Context::TypeResource(TypeResourceContext {
            type_name: name.into(),
            user,
        }),
    )
    .type_resource
    .unwrap()
    .spotlight
    .unwrap()
}

#[test]
fn spotlight_retains_independent_retail_building_history_when_disabled() {
    let disabled = configure(vec![], 0);
    assert_eq!(
        spotlight(disabled.clone(), "SAME", WEAPON_USER_BUILDING),
        SpotlightPolicy {
            has_spotlight: Some(false),
            ..Default::default()
        }
    );
    assert_eq!(
        spotlight(disabled.clone(), "STOCKONLY", WEAPON_USER_BUILDING).has_spotlight,
        Some(true),
        "only a BuildingExtension visit reads the stock capability"
    );
    let enabled = configure(disabled, 1);
    let foot = spotlight(enabled.clone(), "SAME", WEAPON_USER_UNIT);
    assert!(foot.use_type_rules);
    assert_eq!(
        foot.has_spotlight,
        Some(true),
        "earlier disabled pass was retained"
    );
    assert_eq!(
        spotlight(enabled.clone(), "SAME", WEAPON_USER_INFANTRY).has_spotlight,
        Some(false),
        "same-name classes do not share suffix visits"
    );
    let disabled = configure(configure(enabled, 2), 9);
    assert_eq!(
        spotlight(disabled.clone(), "SAME", WEAPON_USER_BUILDING).has_spotlight,
        Some(false),
        "Ares-trimmed true must never replace retail's false history"
    );
    assert_eq!(
        spotlight(disabled.clone(), "STOCKONLY", WEAPON_USER_BUILDING).has_spotlight,
        Some(true)
    );
    assert!(!spotlight(disabled.clone(), "STOCKONLY", WEAPON_USER_BUILDING).use_type_rules);
    assert_eq!(
        spotlight(disabled, "SAME", WEAPON_USER_UNIT).has_spotlight,
        Some(false)
    );
}

#[test]
fn spotlight_freezes_numeric_enum_and_channel_policies_without_live_reads() {
    let first = configure(vec![], 3);
    let policy = spotlight(first.clone(), "SAME", WEAPON_USER_UNIT);
    assert_eq!((policy.start_height, policy.distance), (-20, i32::MAX));
    assert_eq!(policy.attached_to, SpotlightAttachment::Turret);
    assert_eq!(
        policy.glow_flags, 10,
        "red and blue flags keep native bit positions"
    );
    let barrel = configure(first, 4);
    let unknown = configure(barrel, 5);
    let frozen = configure(configure(unknown, 6), 9);
    let policy = spotlight(frozen.clone(), "SAME", WEAPON_USER_UNIT);
    assert_eq!((policy.start_height, policy.distance), (-20, i32::MAX));
    assert_eq!(policy.attached_to, SpotlightAttachment::Barrel);
    assert_eq!(
        policy.glow_flags, 13,
        "color, green and blue survive later missing keys"
    );
    let decoded: Configuration = serde_json::from_slice(&frozen).unwrap();
    let default = decoded.spotlight.query("DEFAULT", WEAPON_USER_UNIT, true);
    assert_eq!(default.start_height, 430);
    assert_eq!(default.distance, 1024);
    assert_eq!(default.attached_to, SpotlightAttachment::Body);
    let json: serde_json::Value = serde_json::from_slice(&frozen).unwrap();
    assert!(
        json["spotlight"]["types"][WEAPON_USER_UNIT.to_string()]
            .get("DEFAULT")
            .is_none()
    );
}
