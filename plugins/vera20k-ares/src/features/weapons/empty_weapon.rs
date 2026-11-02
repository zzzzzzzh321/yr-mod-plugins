//! Ares NoAmmoWeapon / NoAmmoAmount; the host's canonical selector owns order.
//! Ares TechnoClass_SelectWeapon_NoAmmoWeapon10047AE0, installed at6F3410.
//! TechnoTypeExt ctor1003CD6C/CD72 seeds threshold0/index-1; reads1003EEEB/EF01.
use crate::support::{ConfigView, Host};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use vera_mod_api::{ConfigureContext, DepletedWeaponPolicy};

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Configuration {
    weapons: BTreeMap<String, DepletedWeaponPolicy>,
}

impl Configuration {
    pub(crate) fn configure<H: Host>(
        &mut self,
        context: &ConfigureContext,
        view: &ConfigView<'_, H>,
    ) -> Result<(), String> {
        self.weapons.clear();
        for name in &context.object_types {
            let section = view.section_or_empty(name);
            let index = section.read_ares_int("NoAmmoWeapon", -1);
            // Ares10047B08..10047B1D tests only the sign and returns the
            // complete nonnegative index. Every negative value disables this
            // override. Slot lookup/elite fallback and the host's array safety
            // bound belong to GetWeapon, not to configuration or this policy.
            if index >= 0 {
                self.weapons.insert(
                    name.clone(),
                    DepletedWeaponPolicy {
                        weapon_index: index,
                        threshold: section.read_ares_int("NoAmmoAmount", 0),
                    },
                );
            }
        }
        Ok(())
    }

    pub(crate) fn policy(&self, name: &str) -> Option<DepletedWeaponPolicy> {
        self.weapons.get(name).copied()
    }
}
