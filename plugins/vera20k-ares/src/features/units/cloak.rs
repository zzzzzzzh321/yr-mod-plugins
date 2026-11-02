//! Ares TypeExt Cloakable.Allowed: constructor1003C9D7 seeds true and
//! reader1003EBEF retains earlier valid values. The shared host bool reader
//! owns parsing; the host's CloakRuntime remains the only clock/state owner.
use crate::support::{ConfigView, Host};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use vera_mod_api::{
    ConfigureContext, WEAPON_USER_AIRCRAFT, WEAPON_USER_BUILDING, WEAPON_USER_INFANTRY,
    WEAPON_USER_UNIT,
};

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Configuration {
    permissions: BTreeMap<u8, BTreeMap<String, bool>>,
}

impl Configuration {
    pub(crate) fn configure<H: Host>(
        &mut self,
        context: &ConfigureContext,
        view: &ConfigView<'_, H>,
    ) {
        if context.final_pass {
            // The constructor allows every retail type. Its empty sparse
            // map needs no copy of the full registered directory to prune.
            if !self.permissions.is_empty() {
                let names = context
                    .object_types
                    .iter()
                    .map(String::as_str)
                    .collect::<BTreeSet<_>>();
                for members in self.permissions.values_mut() {
                    members.retain(|name, allowed| names.contains(name.as_str()) && !*allowed);
                }
                self.permissions.retain(|_, members| !members.is_empty());
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
                if !section.is_present("Cloakable.Allowed") {
                    continue;
                }
                let allowed =
                    section.read_ares_bool("Cloakable.Allowed", self.allowed(&name, user));
                if allowed {
                    if let Some(members) = self.permissions.get_mut(&user) {
                        members.remove(&name);
                    }
                } else {
                    self.permissions
                        .entry(user)
                        .or_default()
                        .insert(name, false);
                }
            }
        }
        self.permissions.retain(|_, members| !members.is_empty());
    }

    pub(crate) fn allowed(&self, name: &str, user: u8) -> bool {
        self.permissions
            .get(&user)
            .and_then(|members| members.get(name))
            .copied()
            .unwrap_or(true)
    }
}
