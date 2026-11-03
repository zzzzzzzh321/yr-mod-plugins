//! Already-decoded reader replies check composition and retention; canonical
//! numeric, mission and registered-Anim parsing are tested by the host.
use super::*;
use crate::component::tests::{FrozenHost, frozen_query};
use std::cell::Cell;
use vera_mod_api::{
    ConfigurationBoundary, ConfigureBoundaryContext, EmpResourceContext as Query,
    EmpResourcePolicy as Policy, EmpTypePolicy, EmpWarheadPolicy, NullableResourceName as Name,
    ProductionReadPoint, ProductionReadSite, ProductionRegistryView, WEAPON_USER_INFANTRY,
    WEAPON_USER_UNIT,
};

struct EmpHost {
    pass: u8,
    view_calls: Cell<u8>,
}

impl EmpHost {
    fn new(pass: u8) -> Self {
        Self {
            pass,
            view_calls: Cell::new(0),
        }
    }
    fn value(&self, section: &str, key: &str) -> Option<serde_json::Value> {
        use serde_json::json;
        match (self.pass, section, key) {
            (0 | 2, "VeraFeatures", "AresEMP") => Some(json!(false)),
            (1, "VeraFeatures", "AresEMP") => Some(json!(true)),
            (0 | 1, "CombatDamage", "EMPAIRecoverMission") => Some(json!("decoded mission")),
            (0, "SAME", "ImmuneToEMP") => Some(json!(false)),
            (1, "SAME", "ImmuneToEMP") => Some(json!("retain")),
            (0, "SAME", "EMP.Modifier") => Some(json!(0.5f64.to_bits())),
            (0, "SAME", "EMP.Threshold") => Some(json!(-1)),
            (1 | 3, "SAME", "EMP.Threshold") => Some(json!(0)),
            (0..=2, "SAME", "EMP.Sparkles") | (0, "CLEARED", "EMP.Sparkles") => {
                Some(json!("registered reader"))
            }
            (0, "SAME", "VeteranAbilities") => Some(json!(["EMPIMMUNE"])),
            (1, "SAME", "VeteranAbilities") => Some(serde_json::Value::Null),
            (1, "SAME", "EliteAbilities") => Some(json!(["empimmune"])),
            (0, "EMPWH", "EMP.Duration") => Some(json!(3)),
            (1, "EMPWH", "EMP.Duration") => Some(json!("retain")),
            (0, "EMPWH", "EMP.Cap") => Some(json!(2)),
            (1, "EMPWH", "EMP.Cap") => Some(json!(0)),
            (0 | 1, "EMPWH", "EMP.Sparkles") => Some(json!("registered reader")),
            (0, "EMPWH", "AffectsEnemies") => Some(json!(false)),
            (0, "EMPWH", "AffectsOwner") => Some(json!(true)),
            (1, "EMPWH", "AffectsOwner") => Some(json!("retain")),
            _ => None,
        }
    }
}

impl Host for EmpHost {
    fn config(&self, section: &str, key: &str) -> Option<String> {
        self.value(section, key)
            .map(|_| "decoded host fixture".into())
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
            assert_eq!(
                self.view_calls.replace(1),
                0,
                "physical metadata must be shared once per Configure"
            );
            let reads = if self.pass <= 1 {
                vec![
                    ProductionReadPoint {
                        prefix: Default::default(),
                        site: ProductionReadSite::ProcessEntry,
                    },
                    ProductionReadPoint {
                        prefix: Default::default(),
                        site: ProductionReadSite::BeforeTypeData,
                    },
                ]
            } else {
                vec![]
            };
            return serde_json::to_vec(&ProductionRegistryView {
                reads,
                ..Default::default()
            })
            .map_err(|e| e.to_string());
        }
        let result = match (section, key, reader) {
            ("CombatDamage", "EMPAIRecoverMission", "ares-nullable-mission") => {
                if self.pass == 0 {
                    b"1".to_vec()
                } else {
                    current.to_vec()
                }
            }
            (_, "EMP.Sparkles", reader) if reader.starts_with("ares-nullable-ref:") => {
                let value = match (self.pass, section, reader) {
                    (0, "SAME", "ares-nullable-ref:$animations@$vehicles") => {
                        Name::Named("EARLY".into())
                    }
                    (0, "SAME", "ares-nullable-ref:$animations@$infantry") => Name::Inherit,
                    (0, "CLEARED", "ares-nullable-ref:$animations@$vehicles") | (2, "SAME", _) => {
                        Name::Clear
                    }
                    (1, "SAME", _) => serde_json::from_slice(current).unwrap(),
                    _ => panic!("unexpected nullable Anim reader: {section}/{reader}"),
                };
                serde_json::to_vec(&value).unwrap()
            }
            ("EMPWH", "EMP.Sparkles", "ares-ref:$animations@$warheads") => {
                if self.pass == 0 {
                    br#""EARLY""#.to_vec()
                } else {
                    current.to_vec()
                }
            }
            ("SAME", "EMP.Threshold", "ares-emp-threshold") if self.pass == 3 => {
                return Err("threshold reader rejected fixture".into());
            }
            (_, "ImmuneToEMP" | "AffectsOwner", "ares-nullable-bool") if self.pass == 1 => {
                current.to_vec()
            }
            ("EMPWH", "EMP.Duration", "ares-int") if self.pass == 1 => current.to_vec(),
            _ => {
                let expected = match key {
                    "AresEMP" => "bool",
                    "ImmuneToEMP" | "AffectsOwner" => "ares-nullable-bool",
                    "AffectsEnemies" => "ares-bool",
                    "EMP.Modifier" => "ares-double-bits",
                    "EMP.Threshold" => "ares-emp-threshold",
                    "EMP.Duration" | "EMP.Cap" => "ares-int",
                    "VeteranAbilities" | "EliteAbilities" => "comma-list:2048",
                    _ => panic!("unexpected reader: {section}/{key}/{reader}"),
                };
                assert_eq!(reader, expected);
                serde_json::to_vec(&self.value(section, key).unwrap()).unwrap()
            }
        };
        Ok(result)
    }
    fn registry(&self, registry: &str) -> Vec<String> {
        let names: &[&str] = match (self.pass, registry) {
            (0, "$vehicles") => &["SAME", "CLEARED", "INHERITED"],
            (1..=3, "$vehicles") => &["SAME"],
            (0..=2, "$infantry") => &["SAME"],
            (0 | 1, "$warheads") => &["EMPWH"],
            _ => &[],
        };
        names.iter().map(|name| (*name).into()).collect()
    }
    fn damage_kernel(&self, _: &vera_mod_api::DamageKernelInput) -> Result<i32, String> {
        panic!("EMP configuration has no numeric damage kernel")
    }
}

fn request(config: Vec<u8>, pass: u8) -> Request {
    Request {
        api_version: API_VERSION,
        context: Context::Configure(vera_mod_api::ConfigureContext {
            pass: pass.into(),
            source: "decoded-EMP-fixture".into(),
            final_pass: pass == 5,
            object_types: vec!["SAME".into(), "CLEARED".into(), "INHERITED".into()],
            warheads: vec!["EMPWH".into()],
        }),
        config,
        state: vec![],
    }
}

fn configure(config: Vec<u8>, pass: u8) -> Vec<u8> {
    let request = request(config, pass);
    let plan = evaluate(&request, &EmpHost::new(pass)).unwrap();
    plan.validate_for(&request).unwrap();
    plan.config.unwrap()
}

fn query(config: &[u8], context: Query) -> Policy {
    let plan = frozen_query(config, Context::EmpResource(context));
    assert!(plan.state.is_empty() && plan.projection.is_none());
    plan.emp_resource.unwrap()
}

fn typed(config: &[u8], user: u8, name: &str) -> EmpTypePolicy {
    let Policy::Type { policy } = query(
        config,
        Query::Type {
            user,
            name: name.into(),
        },
    ) else {
        panic!("missing Type descriptor")
    };
    policy
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
fn emp_configuration_freezes_class_identity_nullable_names_and_global_lifetimes() {
    let mut initial = serde_json::to_value(decode_config(&[]).unwrap()).unwrap();
    // EMP BeforeTypeData must run independently from this skipped production
    // global phase; both globals survive the same TypeReset boundary.
    initial["production"]["globals"] =
        serde_json::json!({"phase":3,"ai_bypass_build_limit":[true,false,true]});
    let first = configure(serde_json::to_vec(&initial).unwrap(), 0);
    let enabled = configure(configure(first, 1), 5);
    assert_eq!(
        query(&enabled, Query::Rules),
        Policy::Rules {
            enabled: true,
            ai_recover_mission: Some(1),
            reactivation_requires_powered_center: true
        }
    );
    let vehicle = typed(&enabled, WEAPON_USER_UNIT, "SAME");
    assert_eq!(vehicle.sparkles, Name::Named("EARLY".into()));
    assert_eq!(
        typed(&enabled, WEAPON_USER_INFANTRY, "SAME").sparkles,
        Name::Inherit
    );
    assert_eq!(
        typed(&enabled, WEAPON_USER_UNIT, "CLEARED").sparkles,
        Name::Clear
    );
    assert_eq!(
        typed(&enabled, WEAPON_USER_UNIT, "INHERITED"),
        EmpTypePolicy::default()
    );
    assert_eq!(vehicle.immune, Some(false));
    assert_eq!(vehicle.modifier_bits, 0.5f64.to_bits());
    assert_eq!(vehicle.threshold, 0);
    assert!(vehicle.veteran_immune && vehicle.elite_immune);
    assert_eq!(
        query(
            &enabled,
            Query::Warhead {
                name: "EMPWH".into()
            }
        ),
        Policy::Warhead {
            policy: EmpWarheadPolicy {
                duration: 3,
                cap: 0,
                sparkles: Some("EARLY".into()),
                affects_enemies: false,
                affects_owner: Some(true)
            }
        }
    );
    let disabled = configure(enabled, 2);
    assert_eq!(
        query(&disabled, Query::Rules),
        Policy::Rules {
            enabled: false,
            ai_recover_mission: Some(1),
            reactivation_requires_powered_center: false
        }
    );
    assert_eq!(
        typed(&disabled, WEAPON_USER_UNIT, "SAME").sparkles,
        Name::Clear,
        "off still publishes frozen declarations"
    );
    assert!(matches!(
        query(
            &disabled,
            Query::Warhead {
                name: "EMPWH".into()
            }
        ),
        Policy::Warhead {
            policy: EmpWarheadPolicy {
                duration: 3,
                cap: 0,
                ..
            }
        }
    ));
    let reset = boundary(disabled, ConfigurationBoundary::TypeReset);
    assert_eq!(
        query(&reset, Query::Rules),
        Policy::Rules {
            enabled: true,
            ai_recover_mission: Some(1),
            reactivation_requires_powered_center: true
        }
    );
    assert_eq!(
        typed(&reset, WEAPON_USER_UNIT, "CLEARED"),
        EmpTypePolicy::default()
    );
    let preserved: serde_json::Value = serde_json::from_slice(&reset).unwrap();
    assert_eq!(
        preserved["production"]["globals"],
        initial["production"]["globals"]
    );
    let cleared = boundary(reset, ConfigurationBoundary::ScenarioClear);
    assert_eq!(
        query(&cleared, Query::Rules),
        Policy::Rules {
            enabled: true,
            ai_recover_mission: None,
            reactivation_requires_powered_center: true
        }
    );
}

#[test]
fn emp_native_reader_failure_rejects_the_configuration() {
    let error = evaluate(&request(vec![], 3), &EmpHost::new(3)).unwrap_err();
    assert!(error.contains("EMP.Threshold") && error.contains("threshold reader rejected fixture"));
}
