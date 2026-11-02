//! Composition checks use frozen config; host access here would be a regression.
use super::*;
use vera_mod_api::{DamageKernelInput, TypeResourceContext};

struct FrozenHost;
impl Host for FrozenHost {
    fn config(&self, _: &str, _: &str) -> Option<String> {
        panic!("live INI read")
    }
    fn config_read(&self, _: &str, _: &str, _: &str, _: &[u8]) -> Result<Vec<u8>, String> {
        panic!("live INI read")
    }
    fn registry(&self, _: &str) -> Vec<String> {
        panic!("live registry read")
    }
    fn damage_kernel(&self, _: &DamageKernelInput) -> Result<i32, String> {
        panic!("unexpected damage calculation")
    }
}

#[test]
fn ammunition_uses_frozen_configuration_and_excludes_aircraft() {
    let mut config = serde_json::to_value(Configuration::default()).unwrap();
    config["features"]["ares_ammo"] = serde_json::json!(true);
    config["ammunition"] = serde_json::json!({"costs":{"PAIR":2,"FREE":0,"NEGATIVE":-2}});
    let frozen = serde_json::to_vec(&config).unwrap();
    for (name, cost) in [("PAIR", 2), ("FREE", 0), ("NEGATIVE", -2), ("ordinary", 1)] {
        let request = Request {
            api_version: API_VERSION,
            context: Context::WeaponResource(vera_mod_api::WeaponResourceContext {
                weapon_name: name.into(),
            }),
            config: frozen.clone(),
            state: vec![],
        };
        // HostFixture panics on any configuration read: restored policies
        // must use the frozen bytes, never the current work's INI.
        let plan = evaluate(&request, &FrozenHost).unwrap();
        plan.validate_for(&request).unwrap();
        let policy = plan.weapon_resource.unwrap();
        assert_eq!((policy.cost, policy.reload_after_use), (cost, cost > 0));
        assert!(policy.applies_to(vera_mod_api::WEAPON_USER_UNIT));
        assert!(!policy.applies_to(vera_mod_api::WEAPON_USER_AIRCRAFT));
        assert!(plan.state.is_empty() && plan.projection.is_none());
    }
}

#[test]
fn type_policies_are_frozen_independent_and_exclude_aircraft_reload_only() {
    let mut config = serde_json::to_value(Configuration::default()).unwrap();
    config["features"]["ares_reload"] = serde_json::json!(true);
    config["features"]["ares_no_ammo_weapon"] = serde_json::json!(true);
    config["reload"] = serde_json::json!({"amounts":{"TANK":{"normal_amount":2,"empty_amount":3}}});
    config["empty_weapon"] =
        serde_json::json!({"weapons":{"TANK":{"weapon_index":1,"threshold":1}}});
    let mut request = Request {
        api_version: API_VERSION,
        context: Context::TypeResource(TypeResourceContext {
            type_name: "TANK".into(),
            user: vera_mod_api::WEAPON_USER_UNIT,
        }),
        config: serde_json::to_vec(&config).unwrap(),
        state: vec![],
    };
    let plan = evaluate(&request, &FrozenHost).unwrap();
    plan.validate_for(&request).unwrap();
    let policy = plan.type_resource.unwrap();
    assert_eq!(policy.reload.unwrap().empty_amount, 3);
    assert_eq!(policy.depleted_weapon.unwrap().threshold, 1);
    request.context = Context::TypeResource(TypeResourceContext {
        type_name: "TANK".into(),
        user: vera_mod_api::WEAPON_USER_AIRCRAFT,
    });
    let plane = evaluate(&request, &FrozenHost).unwrap();
    plane.validate_for(&request).unwrap();
    assert!(plane.type_resource.unwrap().reload.is_none());
    assert!(plane.type_resource.unwrap().depleted_weapon.is_some());
    config["features"]["ares_reload"] = serde_json::json!(false);
    config["features"]["ares_no_ammo_weapon"] = serde_json::json!(false);
    request.config = serde_json::to_vec(&config).unwrap();
    assert!(
        evaluate(&request, &FrozenHost)
            .unwrap()
            .type_resource
            .is_none()
    );
}
