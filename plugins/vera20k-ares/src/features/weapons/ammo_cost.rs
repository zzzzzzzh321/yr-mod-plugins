//! Ares weapon Ammo policy, outside the engine. Existing engine resource owners
//! enforce admission, perform the debit and run the shared reload clock.
//! Source: Ares WeaponTypeExt ctor100549B4 (+D0=1), ReadINI10055898;
//! CanFire_Ammo100479B0, TechnoClass_Fire_Ammo100479F0 (calls debit100441F0),
//! InfantryClass_Fire_Ammo10047910 and UnitClass_Fire_Ammo10047B30.

use crate::support::{ConfigView, Host};
use std::collections::BTreeMap;
use vera_mod_api::{WEAPON_USER_BUILDING, WEAPON_USER_INFANTRY, WEAPON_USER_UNIT, WeaponResourcePolicy};

/// The native merged-section reader replays retained physical values. Failed
/// or empty later reads keep the prior successful cost; no parser is copied.
#[derive(Default, serde::Serialize, serde::Deserialize)]
pub(crate) struct Ammunition {
    costs: BTreeMap<String, i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    instance_costs: Option<Vec<i32>>,
}

impl Ammunition {
    pub fn configure<H: Host>(&mut self, host: &H, view: &ConfigView<'_, H>) {
        self.costs.clear();
        for name in host.registry("$weapons") {
            let section = view.section_or_empty(&name);
            if section.is_present("Ammo") {
                let cost = section.read_ares_int("Ammo", 1);
                if cost != 1 {
                    self.costs.insert(name, cost);
                }
            }
        }
    }

    pub(crate) fn configure_instances<H: Host>(
        &mut self,
        view: &ConfigView<'_, H>,
        registry: &vera_mod_api::WeaponRegistryView,
    ) -> Result<(), String> {
        // WeaponTypeExt ctor100549B4 starts Ammo at1; reader10055898
        // follows Owner+24/HasSection10055520 on this exact Weapon object.
        self.instance_costs = Some(
            registry
                .weapons
                .iter()
                .map(|weapon| {
                    view.retained_weapon_instance_read(
                        weapon.identity,
                        &weapon.name,
                        "Ammo",
                        "ares-int",
                        1,
                    )
                })
                .collect::<Result<_, _>>()?,
        );
        self.costs.clear();
        Ok(())
    }

    pub(crate) fn validate_instances(&self, count: usize) -> Result<(), String> {
        if self
            .instance_costs
            .as_ref()
            .map_or(count != 0, |values| values.len() != count)
        {
            return Err("Ammo instance configuration differs from WeaponType catalog".into());
        }
        Ok(())
    }

    pub(crate) fn instance_policy(
        &self,
        identity: vera_mod_api::WeaponTypeRef,
    ) -> Result<WeaponResourcePolicy, String> {
        let cost = self
            .instance_costs
            .as_ref()
            .and_then(|values| values.get(identity.ordinal as usize))
            .copied()
            .ok_or("missing WeaponType Ammo instance")?;
        Ok(Self::policy_for_cost(cost))
    }

    pub fn policy(&self, weapon: &str) -> WeaponResourcePolicy {
        Self::policy_for_cost(self.costs.get(weapon).copied().unwrap_or(1))
    }

    fn policy_for_cost(cost: i32) -> WeaponResourcePolicy {
        WeaponResourcePolicy {
            cost,
            reload_after_use: cost > 0,
            // https://ares-developers.github.io/Ares-docs/new/weapons/ammo.html
            // explicitly excludes AircraftTypes; release retains its own debit.
            users: WEAPON_USER_UNIT | WEAPON_USER_INFANTRY | WEAPON_USER_BUILDING,
            // Ares GetROF_BuildingHack10047A60, installed6FCFA4, skips the
            // loaded-building one-frame shortcut. The host still owns normal
            // ROF, burst, modifiers, random draws and the rearm timer.
            use_weapon_rof: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct CaseHost;
    impl Host for CaseHost {
        fn config(&self, _: &str, _: &str) -> Option<String> {
            Some("present".into())
        }
        fn config_read(
            &self,
            section: &str,
            _: &str,
            _: &str,
            _: &[u8],
        ) -> Result<Vec<u8>, String> {
            Ok(if section == "Gun" {
                b"2".to_vec()
            } else {
                b"3".to_vec()
            })
        }
        fn registry(&self, name: &str) -> Vec<String> {
            match name {
                "$sections" => vec!["Gun".into(), "gun".into()],
                "$weapons" => vec!["Gun".into()],
                _ => vec![],
            }
        }
        fn damage_kernel(&self, _: &vera_mod_api::DamageKernelInput) -> Result<i32, String> {
            panic!("no damage calculation during configure")
        }
    }

    #[test]
    fn case_sensitive_weapon_section_is_not_overwritten_by_an_unrelated_alias() {
        let mut ammo = Ammunition::default();
        let view = ConfigView::new(&CaseHost);
        ammo.configure(&CaseHost, &view);
        view.finish().unwrap();
        assert_eq!(ammo.policy("Gun").cost, 2);
    }
}
