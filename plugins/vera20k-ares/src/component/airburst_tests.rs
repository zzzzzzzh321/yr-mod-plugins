//! Predecoded host values exercise the real configuration and frozen query
//! entrypoints; parser, geometry, RNG and Bullet lifetime stay with engine tests.
use super::*;
use crate::component::tests::{FrozenHost, empty_super_sources, frozen_query};
use vera_mod_api::{
    AirburstPolicy, AirburstTailPolicy, ConfigurationBoundary, ConfigureBoundaryContext,
    ConfigureContext, DamageKernelInput, MatchRulesContext, ProjectileResourceContext,
};

struct AirburstHost {
    physical: bool,
}

impl Host for AirburstHost {
    fn config(&self, section: &str, _: &str) -> Option<String> {
        assert!(
            !["SHOT", "shot", "Default"].contains(&section),
            "no flat Projectile source"
        );
        None
    }

    fn config_read(
        &self,
        section: &str,
        key: &str,
        reader: &str,
        current: &[u8],
    ) -> Result<Vec<u8>, String> {
        if reader.starts_with("projectile-effective:") {
            assert!(
                self.physical,
                "final binding cannot reopen Projectile history"
            );
            // Canonical results can retain arbitrary binary64 bits and a
            // nullable false. No INI parsing or numeric conversion is modeled.
            let value = match key {
                "AirburstSpread" | "RetargetAccuracy" => {
                    assert_eq!(reader, "projectile-effective:ares-double-bits");
                    let bits: u64 = serde_json::from_slice(current).unwrap();
                    assert_eq!(
                        bits,
                        if key == "AirburstSpread" {
                            0x3ff8_0000_0000_0000
                        } else {
                            0
                        }
                    );
                    if section == "SHOT" {
                        serde_json::to_vec(&if key == "AirburstSpread" {
                            0x7ff8_0000_0000_1234u64
                        } else {
                            0x8000_0000_0000_0000u64
                        })
                        .unwrap()
                    } else {
                        current.to_vec()
                    }
                }
                "Splits" => {
                    assert_eq!(
                        (reader, current),
                        ("projectile-effective:ares-bool", &b"false"[..])
                    );
                    if section != "Default" {
                        b"true".to_vec()
                    } else {
                        current.to_vec()
                    }
                }
                "RetargetSelf" => {
                    assert_eq!(
                        (reader, current),
                        ("projectile-effective:ares-bool", &b"true"[..])
                    );
                    if section == "SHOT" {
                        b"false".to_vec()
                    } else {
                        current.to_vec()
                    }
                }
                "AroundTarget" => {
                    assert_eq!(
                        (reader, current),
                        ("projectile-effective:ares-nullable-bool", &b"null"[..])
                    );
                    if section == "SHOT" {
                        b"false".to_vec()
                    } else {
                        current.to_vec()
                    }
                }
                _ => panic!("unexpected Projectile field"),
            };
            return Ok(value);
        }
        if let Some(reply) = empty_super_sources(section, key, reader, current) {
            return reply;
        }
        match reader {
            "rules-registry-view" => {
                serde_json::to_vec(&vera_mod_api::ProductionRegistryView::default())
            }
            "faction-registry-view" => {
                serde_json::to_vec(&vera_mod_api::FactionRegistryView::default())
            }
            _ => return Ok(current.to_vec()),
        }
        .map_err(|e| e.to_string())
    }

    fn registry(&self, section: &str) -> Vec<String> {
        if matches!(section, "$projectile-config" | "$weapon-config") {
            assert!(self.physical, "final binding has no physical type registry");
            if section == "$projectile-config" {
                return ["SHOT", "shot", "Default"]
                    .into_iter()
                    .map(str::to_owned)
                    .collect();
            }
        }
        vec![]
    }

    fn damage_kernel(&self, _: &DamageKernelInput) -> Result<i32, String> {
        panic!("configuration never applies damage")
    }
}

fn projectile_query(name: &str) -> Context {
    Context::ProjectileResource(ProjectileResourceContext {
        projectile_name: name.into(),
    })
}

fn policy(config: &[u8], name: &str) -> AirburstPolicy {
    let plan = frozen_query(config, projectile_query(name));
    assert!(plan.state.is_empty() && plan.config.is_none() && plan.projection.is_none());
    plan.projectile_resource.unwrap().airburst.unwrap()
}

fn tail(config: &[u8]) -> Option<AirburstTailPolicy> {
    frozen_query(config, Context::MatchRules(MatchRulesContext {}))
        .match_rules
        .unwrap()
        .airburst_tail
}

fn configure(config: &[u8], pass: u64, final_pass: bool) -> Vec<u8> {
    let request = Request {
        api_version: API_VERSION,
        context: Context::Configure(ConfigureContext {
            pass,
            source: "decoded-airburst-phase".into(),
            final_pass,
            object_types: vec![],
            warheads: vec![],
        }),
        config: config.to_vec(),
        state: vec![],
    };
    let plan = evaluate(
        &request,
        &AirburstHost {
            physical: !final_pass,
        },
    )
    .unwrap();
    plan.validate_for(&request).unwrap();
    plan.config.unwrap()
}

#[test]
fn airburst_defaults_and_typed_reads_preserve_bits_identity_and_nullable_false() {
    assert_eq!(
        policy(&[], "Unseen"),
        AirburstPolicy {
            splits: false,
            spread_bits: 0x3ff8_0000_0000_0000,
            retarget_accuracy_bits: 0,
            retarget_self: true,
            around_target: false,
        }
    );
    let config = configure(&[], 0, false);
    assert_eq!(
        policy(&config, "SHOT"),
        AirburstPolicy {
            splits: true,
            spread_bits: 0x7ff8_0000_0000_1234,
            retarget_accuracy_bits: 0x8000_0000_0000_0000,
            retarget_self: false,
            around_target: false,
        }
    );
    let lower = policy(&config, "shot");
    assert!(lower.splits && lower.around_target && lower.retarget_self);
    assert_eq!(lower.spread_bits, 0x3ff8_0000_0000_0000);
    assert_eq!(policy(&config, "Default"), policy(&[], "Unseen"));
    let stored: serde_json::Value = serde_json::from_slice(&config).unwrap();
    assert!(stored["airburst"]["policies"].get("Default").is_none());
    assert!(manifest().hooks.contains(&HookKind::ProjectileResource));
}

#[test]
fn airburst_physical_history_is_retained_without_final_registry_or_live_query_reads() {
    let first = configure(&[], 0, false);
    // The second canonical reply represents invalid-later-input retention.
    let later = configure(&first, 1, false);
    let frozen = configure(&later, 2, true);
    assert_eq!(policy(&first, "SHOT"), policy(&later, "SHOT"));
    assert_eq!(policy(&later, "SHOT"), policy(&frozen, "SHOT"));
    assert_eq!(tail(&frozen), Some(AirburstTailPolicy::AresReplacement));
}

#[test]
fn airburst_historical_omission_waits_for_reset_and_rejects_mixed_cohorts() {
    let historical = serde_json::to_vec(&Configuration {
        firing: Some(firing::Configuration::default()),
        projectile_range: Some(projectile_range::Configuration::default()),
        ..Default::default()
    })
    .unwrap();
    let physical = configure(&historical, 0, false);
    let frozen = configure(&physical, 1, true);
    let old = frozen_query(&frozen, projectile_query("SHOT"));
    assert!(old.projectile_resource.is_none());
    assert!(
        serde_json::to_value(old)
            .unwrap()
            .get("projectile_resource")
            .is_none()
    );
    assert!(
        serde_json::from_slice::<serde_json::Value>(&frozen)
            .unwrap()
            .get("airburst")
            .is_none()
    );
    assert_eq!(
        tail(&frozen),
        Some(AirburstTailPolicy::UnsupportedReplacement)
    );
    for boundary in [
        ConfigurationBoundary::ScenarioClear,
        ConfigurationBoundary::TypeReset,
    ] {
        let request = Request {
            api_version: API_VERSION,
            context: Context::ConfigureBoundary(ConfigureBoundaryContext { boundary }),
            config: frozen.clone(),
            state: vec![],
        };
        let reset = evaluate(&request, &FrozenHost).unwrap();
        reset.validate_for(&request).unwrap();
        let fresh = reset.config.unwrap();
        assert_eq!(policy(&fresh, "SHOT"), policy(&[], "Unseen"));
        assert_eq!(tail(&fresh), Some(AirburstTailPolicy::AresReplacement));
    }
    let request = Request {
        api_version: API_VERSION,
        context: projectile_query("SHOT"),
        state: vec![],
        config: serde_json::to_vec(&Configuration {
            firing: Some(firing::Configuration::default()),
            airburst: Some(airburst::Configuration::default()),
            ..Default::default()
        })
        .unwrap(),
    };
    assert!(
        evaluate(&request, &FrozenHost)
            .unwrap_err()
            .contains("requires the projectile range domain")
    );
}
