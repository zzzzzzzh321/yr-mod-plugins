//! Common Techno suffix716123/716132: TurretROT1003F267/10008CC0,
//! then Convert.Script1003F3F3/1003FA40. Host readers own signed numeric
//! parsing and Find10007EA0 over the reached common construction prefix.
use crate::support::{ConfigView, Host};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use vera_mod_api::{
    ConfigureContext, TypeConversionPolicy, WEAPON_USER_AIRCRAFT, WEAPON_USER_BUILDING,
    WEAPON_USER_INFANTRY, WEAPON_USER_UNIT,
};

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Configuration {
    types: BTreeMap<u8, BTreeMap<String, TypeConversionPolicy>>,
}

impl Configuration {
    pub(crate) fn configure<H: Host>(
        &mut self,
        context: &ConfigureContext,
        view: &ConfigView<'_, H>,
    ) {
        if context.final_pass {
            if !self.types.is_empty() {
                let names: BTreeSet<_> = context.object_types.iter().map(String::as_str).collect();
                for types in self.types.values_mut() {
                    types.retain(|name, _| names.contains(name.as_str()));
                }
                self.types.retain(|_, types| !types.is_empty());
            }
            return;
        }
        for (user, registry) in [
            (WEAPON_USER_BUILDING, "$buildings"),
            (WEAPON_USER_AIRCRAFT, "$aircraft"),
            (WEAPON_USER_UNIT, "$vehicles"),
            (WEAPON_USER_INFANTRY, "$infantry"),
        ] {
            for name in view.registered_names(registry) {
                let section = view.section_in_registry(&name, registry);
                let mut policy = self
                    .types
                    .get_mut(&user)
                    .and_then(|types| types.remove(&name))
                    .unwrap_or_default();
                policy.secondary_rot =
                    section.read_ares_nullable_int("TurretROT", policy.secondary_rot);
                // Do not select by the source class. A cross-class target is
                // a valid declaration; the existing converter rejects it at
                // runtime. Unknown names retain and native none clears.
                policy.script_target =
                    section.read_ares_registered_type("Convert.Script", policy.script_target);
                if policy != TypeConversionPolicy::default() {
                    self.types.entry(user).or_default().insert(name, policy);
                }
            }
        }
        self.types.retain(|_, types| !types.is_empty());
    }

    pub(crate) fn query(&self, name: &str, user: u8, enabled: bool) -> TypeConversionPolicy {
        if enabled {
            self.types
                .get(&user)
                .and_then(|types| types.get(name))
                .copied()
                .unwrap_or_default()
        } else {
            TypeConversionPolicy::default()
        }
    }
}
