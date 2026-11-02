//! Ares ReloadAmount / EmptyReloadAmount configuration and immutable policy.
//! Ares TechnoTypeExt ctor1003C710 (1003CD56..CD66): +458=1, nullable unset;
//! ReadINI1003EEB4..EEDB uses retained Ares Valueable/Nullable integer reads.
//! Reload_ReloadAmount10047A70 chooses the empty override only at count zero.
use crate::support::{ConfigView, Host};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use vera_mod_api::{ConfigureContext, ReloadPolicy};

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Configuration {
    amounts: BTreeMap<String, ReloadPolicy>,
}

impl Configuration {
    pub(crate) fn configure<H: Host>(
        &mut self,
        context: &ConfigureContext,
        view: &ConfigView<'_, H>,
    ) {
        self.amounts.clear();
        for name in &context.object_types {
            let section = view.section_or_empty(name);
            let normal_amount = section.read_ares_int("ReloadAmount", 1);
            let empty_amount = section.read_ares_int("EmptyReloadAmount", normal_amount);
            if normal_amount != 1 || empty_amount != 1 {
                self.amounts.insert(
                    name.clone(),
                    ReloadPolicy {
                        normal_amount,
                        empty_amount,
                    },
                );
            }
        }
    }

    pub(crate) fn policy(&self, name: &str) -> ReloadPolicy {
        self.amounts.get(name).copied().unwrap_or(ReloadPolicy {
            normal_amount: 1,
            empty_amount: 1,
        })
    }
}
