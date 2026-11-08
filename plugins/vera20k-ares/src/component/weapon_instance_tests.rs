//! Decoded retained reader results exercise the real configuration and frozen
//! query entry points. Native parsing and factory order are host concerns.
use super::*;
use crate::component::tests::{FrozenHost, empty_super_sources};
use vera_mod_api::{
    ConfigureContext, DamageKernelInput, WeaponInstanceResourceContext, WeaponReadValue,
    WeaponRegisteredType, WeaponRegistryView, WeaponTypeRef,
};

const STORED: &str = "ABCDEFGHIJKLMNOPQRSTUVWX";

struct InstanceHost {
    physical: bool,
}
impl Host for InstanceHost {
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
        if reader == "weapon-registry-view" {
            assert!(
                self.physical,
                "final query cannot reopen physical instances"
            );
            return serde_json::to_vec(&WeaponRegistryView {
                weapons: [STORED, STORED, "Short"]
                    .iter()
                    .enumerate()
                    .map(|(ordinal, name)| WeaponRegisteredType {
                        identity: WeaponTypeRef {
                            ordinal: ordinal as u32,
                        },
                        name: (*name).into(),
                    })
                    .collect(),
            })
            .map_err(|e| e.to_string());
        }
        if let Some(reader) = reader.strip_prefix("weapon-instance-effective:") {
            assert!(self.physical);
            let input: WeaponReadValue<serde_json::Value> =
                serde_json::from_slice(current).map_err(|e| e.to_string())?;
            assert_eq!(
                [STORED, STORED, "Short"][input.identity.ordinal as usize],
                section
            );
            // Slot1 represents an allocated object which has not read INI.
            let value = match (input.identity.ordinal, key, reader) {
                (0, "Ammo", "ares-int") => serde_json::json!(3),
                (0, "ApplyDamage", "ares-nullable-bool") => serde_json::json!(false),
                (0, "ProjectileRange", "ares-leptons") => serde_json::json!(512),
                (1, "Ammo", "ares-int")
                | (1, "ApplyDamage", "ares-nullable-bool")
                | (1, "ProjectileRange", "ares-leptons") => input.current,
                (2, "Ammo", "ares-int") => serde_json::json!(0),
                (2, "ApplyDamage", "ares-nullable-bool") => serde_json::json!(true),
                (2, "ProjectileRange", "ares-leptons") => serde_json::json!(0),
                _ => panic!("unexpected scalar fixture input"),
            };
            return serde_json::to_vec(&value).map_err(|e| e.to_string());
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

    fn registry(&self, _: &str) -> Vec<String> {
        vec![]
    }
    fn damage_kernel(&self, _: &DamageKernelInput) -> Result<i32, String> {
        panic!("immutable declarations do not calculate damage")
    }
}

fn configure(config: Vec<u8>, final_pass: bool) -> Vec<u8> {
    let request = Request {
        api_version: API_VERSION,
        context: Context::Configure(ConfigureContext {
            pass: u64::from(final_pass),
            source: "instance-source".into(),
            final_pass,
            object_types: vec![],
            warheads: vec![],
        }),
        config,
        state: vec![],
    };
    let plan = evaluate(
        &request,
        &InstanceHost {
            physical: !final_pass,
        },
    )
    .unwrap();
    plan.validate_for(&request).unwrap();
    plan.config.unwrap()
}

fn request(config: &[u8], ordinal: u32, name: &str) -> Request {
    Request {
        api_version: API_VERSION,
        config: config.into(),
        state: vec![],
        context: Context::WeaponInstanceResource(WeaponInstanceResourceContext {
            identity: WeaponTypeRef { ordinal },
            weapon_name: name.into(),
            animation_count: 8,
            is_sonic: false,
            use_fire_particles: false,
        }),
    }
}

#[test]
fn weapon_instances_keep_same_name_defaults_zero_and_frozen_queries() {
    let physical = configure(vec![], false);
    let config = configure(physical.clone(), true);
    let first = evaluate(&request(&config, 0, STORED), &FrozenHost).unwrap();
    first.validate_for(&request(&config, 0, STORED)).unwrap();
    let first = first.weapon_instance_resource.unwrap();
    assert_eq!(first.resource.unwrap().cost, 3);
    assert!(!first.firing.unwrap().apply_damage);
    assert_eq!(first.firing.unwrap().projectile_range, Some(512));
    let unread = evaluate(&request(&config, 1, STORED), &FrozenHost)
        .unwrap()
        .weapon_instance_resource
        .unwrap();
    assert_eq!(unread.resource.unwrap().cost, 1);
    assert!(unread.firing.unwrap().apply_damage);
    assert_eq!(unread.firing.unwrap().projectile_range, Some(100_000));
    let zero_request = request(&config, 2, "Short");
    let zero = evaluate(&zero_request, &FrozenHost).unwrap();
    zero.validate_for(&zero_request).unwrap();
    assert!(zero.config.is_none() && zero.state.is_empty());
    let zero = zero.weapon_instance_resource.unwrap();
    assert_eq!(zero.resource.unwrap().cost, 0);
    assert_eq!(zero.firing.unwrap().projectile_range, Some(0));
    assert!(evaluate(&request(&config, 0, "Short"), &FrozenHost).is_err());
    assert!(evaluate(&request(&config, 3, "Short"), &FrozenHost).is_err());
    let legacy = Request {
        context: Context::WeaponResource(vera_mod_api::WeaponResourceContext {
            weapon_name: STORED.into(),
        }),
        ..zero_request
    };
    assert!(
        evaluate(&legacy, &FrozenHost).is_err(),
        "a name cannot select either colliding instance"
    );
}

#[test]
fn historical_weapon_configuration_does_not_backfill_instance_ownership() {
    let historical = serde_json::to_vec(&Configuration::default()).unwrap();
    let historical = configure(historical, false);
    let legacy_request = request(&historical, 0, STORED);
    let legacy = evaluate(&legacy_request, &FrozenHost).unwrap();
    legacy.validate_for(&legacy_request).unwrap();
    assert!(legacy.weapon_instance_resource.is_none());
    let reset_request = Request {
        api_version: API_VERSION,
        context: Context::ConfigureBoundary(vera_mod_api::ConfigureBoundaryContext {
            boundary: vera_mod_api::ConfigurationBoundary::TypeReset,
        }),
        config: historical,
        state: vec![],
    };
    let reset = evaluate(&reset_request, &FrozenHost).unwrap();
    let fresh = configure(reset.config.unwrap(), false);
    assert!(
        evaluate(&request(&fresh, 0, STORED), &FrozenHost)
            .unwrap()
            .weapon_instance_resource
            .is_some()
    );
}
