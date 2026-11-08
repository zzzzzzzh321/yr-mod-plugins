//! Installed Ares Airburst inputs, frozen from each actual Projectile read.
use crate::support::{ConfigView, Host};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use vera_mod_api::{AirburstPolicy, ProjectileResourcePolicy};

// Ares BulletTypeExt constructor100192C0, writes10019317..1001932C.
// AroundTarget has no presence initially and resolves to the final Splits value.
const DEFAULT_POLICY: AirburstPolicy = AirburstPolicy {
    splits: false,
    spread_bits: 0x3ff8_0000_0000_0000,
    retarget_accuracy_bits: 0,
    retarget_self: true,
    around_target: false,
};

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Configuration {
    policies: BTreeMap<String, AirburstPolicy>,
}

impl Configuration {
    pub(crate) fn configure<H: Host>(&mut self, host: &H) -> Result<(), String> {
        self.policies.clear();
        let view = ConfigView::native_only(host);
        for name in host.registry("$projectile-config") {
            // LoadFromINI100193C0 reads in this order. The host replays the
            // retained Projectile history using the shared native readers;
            // there is no latest flat INI source, float parser or clamp here.
            let spread_bits = view.retained_projectile_read(
                &name,
                "AirburstSpread",
                "ares-double-bits",
                DEFAULT_POLICY.spread_bits,
            )?;
            let retarget_accuracy_bits = view.retained_projectile_read(
                &name,
                "RetargetAccuracy",
                "ares-double-bits",
                DEFAULT_POLICY.retarget_accuracy_bits,
            )?;
            let splits =
                view.retained_projectile_read(&name, "Splits", "ares-bool", DEFAULT_POLICY.splits)?;
            let retarget_self = view.retained_projectile_read(
                &name,
                "RetargetSelf",
                "ares-bool",
                DEFAULT_POLICY.retarget_self,
            )?;
            let around_target = view.retained_projectile_read::<Option<bool>>(
                &name,
                "AroundTarget",
                "ares-nullable-bool",
                None,
            )?;
            let policy = AirburstPolicy {
                splits,
                spread_bits,
                retarget_accuracy_bits,
                retarget_self,
                // Ares DetonateAt_Splits1001A0FA..1001A127 resolves presence.
                around_target: around_target.unwrap_or(splits),
            };
            if policy != DEFAULT_POLICY {
                self.policies.insert(name, policy);
            }
        }
        view.finish()
    }

    pub(crate) fn query(&self, projectile_name: &str) -> ProjectileResourcePolicy {
        ProjectileResourcePolicy {
            airburst: Some(
                self.policies
                    .get(projectile_name)
                    .copied()
                    .unwrap_or(DEFAULT_POLICY),
            ),
            ..Default::default()
        }
    }
}
