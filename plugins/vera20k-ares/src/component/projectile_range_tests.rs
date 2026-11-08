//! Decoded native read results enter the real component lifecycle. Parsing and
//! live travel-budget consumption are checked by the paired engine's tests.
use super::*;
use crate::component::tests::{FrozenHost, empty_super_sources, frozen_query};
use vera_mod_api::{
    ConfigurationBoundary, ConfigureBoundaryContext, ConfigureContext, DamageKernelInput,
    MatchRulesContext, WeaponFiringResourceContext,
};

struct RangeHost {
    physical: bool,
}

impl Host for RangeHost {
    fn config(&self, _: &str, _: &str) -> Option<String> {
        None
    }

    fn config_read(
        &self,
        section: &str,
        key: &str,
        reader: &str,
        current: &[u8],
    ) -> Result<Vec<u8>, String> {
        if reader.starts_with("weapon-effective:") {
            assert!(
                self.physical,
                "final binding cannot reopen physical weapons"
            );
            return Ok(match reader {
                "weapon-effective:ares-leptons" => {
                    assert_eq!((key, current), ("ProjectileRange", &b"100000"[..]));
                    // Supplied canonical results, including invalid-later-value
                    // retention. This fixture does not parse or convert INI.
                    match section {
                        "Gun" => b"1537".to_vec(),
                        "gun" => b"-2147483648".to_vec(),
                        "Zero" => b"0".to_vec(),
                        _ => current.to_vec(),
                    }
                }
                "weapon-effective:ares-nullable-bool" => {
                    assert_eq!((key, current), ("ApplyDamage", &b"null"[..]));
                    current.to_vec()
                }
                _ => panic!("unexpected weapon reader"),
            });
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
        if section == "$weapon-config" {
            assert!(
                self.physical,
                "final binding has no physical weapon registry"
            );
            return ["Gun", "gun", "Zero", "Default"]
                .into_iter()
                .map(str::to_owned)
                .collect();
        }
        vec![]
    }

    fn damage_kernel(&self, _: &DamageKernelInput) -> Result<i32, String> {
        panic!("configuration does not calculate damage")
    }
}

fn firing_query(name: &str) -> Context {
    Context::WeaponFiringResource(WeaponFiringResourceContext {
        weapon_name: name.into(),
        animation_count: 0,
        is_sonic: false,
        use_fire_particles: false,
    })
}

fn range(config: &[u8], name: &str) -> Option<i32> {
    let plan = frozen_query(config, firing_query(name));
    assert!(plan.state.is_empty() && plan.config.is_none());
    plan.weapon_firing_resource.unwrap().projectile_range
}

fn fuel(config: &[u8]) -> Option<bool> {
    frozen_query(config, Context::MatchRules(MatchRulesContext {}))
        .match_rules
        .unwrap()
        .projectile_fuel
}

fn configure(config: &[u8], pass: u64, final_pass: bool) -> Vec<u8> {
    let request = Request {
        api_version: API_VERSION,
        context: Context::Configure(ConfigureContext {
            pass,
            source: "decoded-range-phase".into(),
            final_pass,
            object_types: vec![],
            warheads: vec![],
        }),
        config: config.to_vec(),
        state: vec![],
    };
    let plan = evaluate(
        &request,
        &RangeHost {
            physical: !final_pass,
        },
    )
    .unwrap();
    plan.validate_for(&request).unwrap();
    plan.config.unwrap()
}

#[test]
fn range_policy_preserves_signed_host_results_exact_ids_and_constructor_default() {
    let mut domain = projectile_range::Configuration::default();
    domain.configure(&RangeHost { physical: true }).unwrap();
    let config = serde_json::to_vec(&Configuration {
        firing: Some(firing::Configuration::default()),
        projectile_range: Some(domain),
        ..Default::default()
    })
    .unwrap();
    assert_eq!(range(&config, "Gun"), Some(1537));
    assert_eq!(range(&config, "gun"), Some(i32::MIN));
    assert_eq!(range(&config, "Zero"), Some(0));
    assert_eq!(range(&config, "Default"), Some(100_000));
    assert_eq!(range(&config, "Unseen"), Some(100_000));
    assert_eq!(fuel(&config), Some(true));
}

#[test]
fn physical_range_reads_retain_native_history_and_final_binding_only_freezes() {
    let mut config = vec![];
    // The later physical reply represents the native reader retaining the first
    // successful value after invalid input, not a guest-side latest-value parse.
    for (pass, final_pass) in [(0, false), (1, false), (2, true)] {
        config = configure(&config, pass, final_pass);
        assert_eq!(range(&config, "Gun"), Some(1537));
        assert_eq!(fuel(&config), Some(true));
    }
}

#[test]
fn historical_range_omission_survives_configure_until_real_boundary() {
    // 0.29 knows firing but has no range domain. A nonempty frozen config must
    // not opt into the new runtime merely because firing is already present.
    let historical = serde_json::to_vec(&Configuration {
        firing: Some(firing::Configuration::default()),
        ..Default::default()
    })
    .unwrap();
    let physical = configure(&historical, 0, false);
    let frozen = configure(&physical, 1, true);
    assert_eq!(range(&frozen, "Gun"), None);
    assert_eq!(fuel(&frozen), None);
    assert!(
        serde_json::from_slice::<serde_json::Value>(&frozen)
            .unwrap()
            .get("projectile_range")
            .is_none()
    );
    for context in [
        firing_query("Gun"),
        Context::MatchRules(MatchRulesContext {}),
    ] {
        let wire = serde_json::to_value(frozen_query(&frozen, context)).unwrap();
        assert!(
            wire.get("weapon_firing_resource")
                .is_none_or(|policy| policy.get("projectile_range").is_none())
        );
        assert!(
            wire.get("match_rules")
                .is_none_or(|policy| policy.get("projectile_fuel").is_none())
        );
    }
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
        assert_eq!(range(&fresh, "Gun"), Some(100_000));
        assert_eq!(fuel(&fresh), Some(true));
    }
    let malformed = serde_json::to_vec(&Configuration {
        projectile_range: Some(projectile_range::Configuration::default()),
        ..Default::default()
    })
    .unwrap();
    let request = Request {
        api_version: API_VERSION,
        context: firing_query("Gun"),
        config: malformed,
        state: vec![],
    };
    assert!(
        evaluate(&request, &FrozenHost)
            .unwrap_err()
            .contains("requires the firing domain")
    );
}
