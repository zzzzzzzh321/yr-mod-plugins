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
            // Documented Ares slots are primary/secondary. Refuse unsupported
            // configuration instead of mapping an invalid slot to another gun.
            if !matches!(index, -1..=1) {
                return Err(format!("[{name}] NoAmmoWeapon must be -1, 0 or 1"));
            }
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
