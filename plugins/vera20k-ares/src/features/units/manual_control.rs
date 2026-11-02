//! Ares manual input restrictions. The guest requests the host's native bool
//! reader; local action/cursor owners consume the frozen type projection.
//! This policy never rejects scripted/AI simulation orders or changes cargo.
//! Ares3.0 TechnoTypeExt reader1003EFEC/1003F002/1003F018/1003F02E
//! stores Unload/Fire/Enter/SelfGuard through Valueable<bool>1000EBF0.
use crate::support::{ConfigView, Host};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use vera_mod_api::{
    ConfigureContext, InputPolicy, WEAPON_USER_AIRCRAFT, WEAPON_USER_INFANTRY, WEAPON_USER_UNIT,
};

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Configuration {
    policies: BTreeMap<String, InputPolicy>,
}

impl Configuration {
    pub(crate) fn configure<H: Host>(
        &mut self,
        context: &ConfigureContext,
        view: &ConfigView<'_, H>,
    ) {
        self.policies.clear();
        for name in &context.object_types {
            let section = view.section_or_empty(name);
            let policy = InputPolicy {
                deny_manual_attack: section.read_ares_bool("NoManualFire", false),
                deny_manual_unload: section.read_ares_bool("NoManualUnload", false),
                hide_transport_entry: section.read_ares_bool("NoManualEnter", false),
                deny_self_guard_area: section.read_ares_bool("NoSelfGuardArea", false),
            };
            if policy != InputPolicy::default() {
                self.policies.insert(name.clone(), policy);
            }
        }
    }

    pub(crate) fn policy(&self, name: &str, user: u8) -> InputPolicy {
        // Enabled composition owns explicit false decisions too. Otherwise a
        // restored false policy could fall through to a current built-in INI.
        let mut policy = self.policies.get(name).copied().unwrap_or_default();
        if !matches!(user, WEAPON_USER_UNIT | WEAPON_USER_AIRCRAFT) {
            policy.deny_manual_unload = false;
        }
        if user != WEAPON_USER_INFANTRY {
            policy.deny_self_guard_area = false;
        }
        policy
    }
}
