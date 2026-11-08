//! Frozen optional Projectile bounds for the host's existing launch scatter.
use crate::support::{ConfigView, Host};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use vera_mod_api::BallisticScatterPolicy;

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Configuration {
    policies: BTreeMap<String, BallisticScatterPolicy>,
}

impl Configuration {
    pub(crate) fn configure<H: Host>(&mut self, host: &H) -> Result<(), String> {
        self.policies.clear();
        let view = ConfigView::native_only(host);
        for name in host.registry("$projectile-config") {
            // BulletTypeExt constructor10019330..10019339 leaves both absent.
            // LoadFromINI1001949D..100194BE reads Min then Max via10019660;
            // that wrapper only sets presence after a successful Leptons read.
            // The host retains full physical history and the canonical parser.
            let min = view.retained_projectile_read::<Option<i32>>(
                &name,
                "BallisticScatter.Min",
                "ares-nullable-leptons",
                None,
            )?;
            let max = view.retained_projectile_read::<Option<i32>>(
                &name,
                "BallisticScatter.Max",
                "ares-nullable-leptons",
                None,
            )?;
            let policy = BallisticScatterPolicy { min, max };
            if policy != BallisticScatterPolicy::default() {
                self.policies.insert(name, policy);
            }
        }
        view.finish()
    }

    pub(crate) fn query(&self, projectile_name: &str) -> BallisticScatterPolicy {
        self.policies
            .get(projectile_name)
            .copied()
            .unwrap_or_default()
    }
}
