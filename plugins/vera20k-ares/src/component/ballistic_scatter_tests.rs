//! Decoded native reader results enter the component's real configuration and
//! immutable query paths. Parser, launch scatter and RNG are engine concerns.
use super::*;
use crate::component::tests::{FrozenHost, empty_super_sources, frozen_query};
use vera_mod_api::{
    AirburstPolicy, BallisticScatterPolicy, ConfigurationBoundary, ConfigureBoundaryContext,
    ConfigureContext, DamageKernelInput, ProjectileResourceContext, ProjectileResourcePolicy,
};

struct ScatterHost {
    physical_pass: Option<u64>,
}

impl Host for ScatterHost {
    fn config(&self, section: &str, _: &str) -> Option<String> {
        assert!(
            !["SHOT", "shot", "Absent"].contains(&section),
            "no flat Projectile read"
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
            let pass = self
                .physical_pass
                .expect("final binding cannot reread Projectile history");
            if reader == "projectile-effective:ares-nullable-leptons" {
                assert_eq!(current, b"null");
                // These are supplied native results, not text to parse: SHOT
                // retains valid zero/negative after a later invalid read;
                // shot has first-invalid None followed by a successful value.
                let result: Option<i32> = match (section, key) {
                    ("SHOT", "BallisticScatter.Min") => Some(0),
                    ("SHOT", "BallisticScatter.Max") => Some(-37),
                    ("shot", "BallisticScatter.Min") if pass == 0 => None,
                    ("shot", "BallisticScatter.Min") => Some(777),
                    ("shot", "BallisticScatter.Max") => Some(i32::MIN),
                    ("Absent", "BallisticScatter.Min" | "BallisticScatter.Max") => None,
                    _ => panic!("unexpected nullable Leptons read"),
                };
                return serde_json::to_vec(&result).map_err(|e| e.to_string());
            }
            // Existing Airburst reads share the physical directory; preserve
            // a nondefault declaration so composition cannot silently erase it.
            if (section, key, reader) == ("SHOT", "Splits", "projectile-effective:ares-bool") {
                return Ok(b"true".to_vec());
            }
            return Ok(current.to_vec());
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
            assert!(
                self.physical_pass.is_some(),
                "final binding has no physical registry"
            );
            if section == "$projectile-config" {
                return ["SHOT", "shot", "Absent"]
                    .into_iter()
                    .map(str::to_owned)
                    .collect();
            }
        }
        vec![]
    }

    fn damage_kernel(&self, _: &DamageKernelInput) -> Result<i32, String> {
        panic!("configuration does not calculate damage")
    }
}

fn projectile_query(name: &str) -> Context {
    Context::ProjectileResource(ProjectileResourceContext {
        projectile_name: name.into(),
    })
}

fn query(config: &[u8], name: &str) -> Option<ProjectileResourcePolicy> {
    let plan = frozen_query(config, projectile_query(name));
    assert!(plan.config.is_none() && plan.projection.is_none() && plan.state.is_empty());
    plan.projectile_resource
}

fn configure(config: &[u8], pass: u64, final_pass: bool) -> Vec<u8> {
    let request = Request {
        api_version: API_VERSION,
        context: Context::Configure(ConfigureContext {
            pass,
            source: "decoded-scatter-phase".into(),
            final_pass,
            object_types: vec![],
            warheads: vec![],
        }),
        config: config.to_vec(),
        state: vec![],
    };
    let plan = evaluate(
        &request,
        &ScatterHost {
            physical_pass: (!final_pass).then_some(pass),
        },
    )
    .unwrap();
    plan.validate_for(&request).unwrap();
    plan.config.unwrap()
}

#[test]
fn scatter_keeps_nullable_signed_results_exact_ids_and_coexisting_airburst() {
    let config = configure(&[], 0, false);
    let upper = query(&config, "SHOT").unwrap();
    assert_eq!(
        upper.ballistic_scatter,
        Some(BallisticScatterPolicy {
            min: Some(0),
            max: Some(-37)
        })
    );
    assert_eq!(
        upper.airburst,
        Some(AirburstPolicy {
            splits: true,
            spread_bits: 0x3ff8_0000_0000_0000,
            retarget_accuracy_bits: 0,
            retarget_self: true,
            around_target: true,
        })
    );
    assert_eq!(
        query(&config, "shot").unwrap().ballistic_scatter,
        Some(BallisticScatterPolicy {
            min: None,
            max: Some(i32::MIN)
        })
    );
    assert_eq!(
        query(&config, "Absent").unwrap().ballistic_scatter,
        Some(BallisticScatterPolicy::default())
    );
    assert_eq!(
        query(&config, "Unseen").unwrap().ballistic_scatter,
        Some(BallisticScatterPolicy::default())
    );
    let stored: serde_json::Value = serde_json::from_slice(&config).unwrap();
    assert!(
        stored["ballistic_scatter"]["policies"]
            .get("Absent")
            .is_none()
    );
}

#[test]
fn scatter_updates_physical_history_and_freezes_without_final_or_live_source_reads() {
    let first = configure(&[], 0, false);
    let later = configure(&first, 1, false);
    let frozen = configure(&later, 2, true);
    assert_eq!(
        query(&first, "shot")
            .unwrap()
            .ballistic_scatter
            .unwrap()
            .min,
        None
    );
    assert_eq!(
        query(&later, "shot")
            .unwrap()
            .ballistic_scatter
            .unwrap()
            .min,
        Some(777)
    );
    assert_eq!(query(&first, "SHOT"), query(&later, "SHOT"));
    assert_eq!(query(&later, "shot"), query(&frozen, "shot"));
}

#[test]
fn scatter_ownership_is_independent_and_old_configuration_waits_for_real_reset() {
    let historical = serde_json::to_vec(&Configuration {
        firing: Some(firing::Configuration::default()),
        projectile_range: Some(projectile_range::Configuration::default()),
        airburst: Some(airburst::Configuration::default()),
        ..Default::default()
    })
    .unwrap();
    let physical = configure(&historical, 0, false);
    let frozen = configure(&physical, 1, true);
    let old = query(&frozen, "SHOT").unwrap();
    assert!(old.airburst.is_some() && old.ballistic_scatter.is_none());
    assert!(
        serde_json::to_value(old)
            .unwrap()
            .get("ballistic_scatter")
            .is_none()
    );
    assert!(
        serde_json::from_slice::<serde_json::Value>(&frozen)
            .unwrap()
            .get("ballistic_scatter")
            .is_none()
    );
    assert!(
        query(
            &serde_json::to_vec(&Configuration::default()).unwrap(),
            "SHOT"
        )
        .is_none()
    );
    // No Airburst/firing/range ownership or weapon feature flags are required.
    let independent = serde_json::to_vec(&Configuration {
        ballistic_scatter: Some(ballistic_scatter::Configuration::default()),
        ..Default::default()
    })
    .unwrap();
    assert_eq!(
        query(&independent, "SHOT"),
        Some(ProjectileResourcePolicy {
            airburst: None,
            ballistic_scatter: Some(BallisticScatterPolicy::default()),
        })
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
        let fresh = query(&reset.config.unwrap(), "SHOT").unwrap();
        assert!(fresh.airburst.is_some());
        assert_eq!(
            fresh.ballistic_scatter,
            Some(BallisticScatterPolicy::default())
        );
    }
}
