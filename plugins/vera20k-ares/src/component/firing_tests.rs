//! Firing policy tests use decoded host replies; actual native parsing and
//! shot/animation consumption are checked by the engine's component tests.
use super::*;
use crate::component::tests::{FrozenHost, frozen_query};
use vera_mod_api::{
    ConfigurationBoundary, ConfigureBoundaryContext, DamageKernelInput, WeaponFiringResourceContext,
};

fn query(name: &str, count: u32, sonic: bool, fire: bool) -> Context {
    Context::WeaponFiringResource(WeaponFiringResourceContext {
        weapon_name: name.into(),
        animation_count: count,
        is_sonic: sonic,
        use_fire_particles: fire,
    })
}

#[test]
fn firing_defaults_use_complete_animation_bins_and_special_weapon_damage_gate() {
    let ordinary = frozen_query(&[], query("GUN", 17, false, false))
        .weapon_firing_resource
        .unwrap();
    assert!(ordinary.apply_damage);
    assert_eq!(ordinary.muzzle_facings, 16);
    let sonic = frozen_query(&[], query("SONIC", 32, true, false))
        .weapon_firing_resource
        .unwrap();
    assert!(!sonic.apply_damage);
    assert_eq!(sonic.muzzle_facings, 32);
    let fire = frozen_query(&[], query("FIRE", 65_537, false, true))
        .weapon_firing_resource
        .unwrap();
    assert!(!fire.apply_damage);
    assert_eq!(fire.muzzle_facings, 65_536);
    assert_eq!(
        frozen_query(&[], query("SINGLE", 7, false, false))
            .weapon_firing_resource
            .unwrap()
            .muzzle_facings,
        1
    );
    assert_eq!(
        frozen_query(&[], query("EMPTY", 0, false, false))
            .weapon_firing_resource
            .unwrap()
            .muzzle_facings,
        0
    );
}

struct FiringHost;
impl Host for FiringHost {
    fn config(&self, _: &str, _: &str) -> Option<String> {
        panic!("raw latest-value presence cannot decide retained nullable history")
    }
    fn config_read(
        &self,
        section: &str,
        key: &str,
        reader: &str,
        current: &[u8],
    ) -> Result<Vec<u8>, String> {
        assert_eq!(
            (key, reader, current),
            ("ApplyDamage", "ares-nullable-bool", &b"null"[..])
        );
        // Predecoded results include a retained value despite a later invalid
        // physical value. Do not implement a second INI parser in this test.
        Ok(match section {
            "Gun" => b"true".to_vec(),
            "gun" => b"false".to_vec(),
            "Empty" => b"null".to_vec(),
            _ => panic!("unknown weapon read"),
        })
    }
    fn registry(&self, section: &str) -> Vec<String> {
        assert_eq!(section, "$weapons");
        ["Gun", "gun", "Empty"]
            .into_iter()
            .map(str::to_owned)
            .collect()
    }
    fn damage_kernel(&self, _: &DamageKernelInput) -> Result<i32, String> {
        panic!("no live damage service while selecting immutable firing policy")
    }
}

#[test]
fn firing_configuration_keeps_nullable_host_result_and_exact_weapon_identity() {
    let mut firing = firing::Configuration::default();
    firing.configure(&FiringHost).unwrap();
    let bytes = serde_json::to_vec(&Configuration {
        firing: Some(firing),
        ..Default::default()
    })
    .unwrap();
    assert!(
        frozen_query(&bytes, query("Gun", 16, true, false))
            .weapon_firing_resource
            .unwrap()
            .apply_damage
    );
    assert!(
        !frozen_query(&bytes, query("gun", 16, false, false))
            .weapon_firing_resource
            .unwrap()
            .apply_damage
    );
    assert!(
        !frozen_query(&bytes, query("Empty", 16, true, false))
            .weapon_firing_resource
            .unwrap()
            .apply_damage
    );
}

#[test]
fn historical_firing_configuration_stays_unowned_until_real_configuration_boundary() {
    let mut historical = serde_json::to_value(Configuration::default()).unwrap();
    historical.as_object_mut().unwrap().remove("firing");
    let bytes = serde_json::to_vec(&historical).unwrap();
    let old = frozen_query(&bytes, query("GUN", 16, false, false));
    assert!(old.weapon_firing_resource.is_none());
    assert!(old.projection.is_none() && old.state.is_empty());
    let request = Request {
        api_version: API_VERSION,
        context: Context::ConfigureBoundary(ConfigureBoundaryContext {
            boundary: ConfigurationBoundary::TypeReset,
        }),
        config: bytes,
        state: vec![],
    };
    let reset = evaluate(&request, &FrozenHost).unwrap();
    reset.validate_for(&request).unwrap();
    let fresh = frozen_query(&reset.config.unwrap(), query("GUN", 16, false, false));
    assert_eq!(fresh.weapon_firing_resource.unwrap().muzzle_facings, 16);
    assert!(manifest().hooks.contains(&HookKind::WeaponFiringResource));
}
