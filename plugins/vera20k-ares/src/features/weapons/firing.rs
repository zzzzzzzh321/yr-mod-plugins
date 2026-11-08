//! Immutable Ares firing decisions. The host retains the animation references,
//! coordinates, damage stages, muzzle constructor and ordinary AI estimate.
use crate::support::{ConfigView, Host};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use vera_mod_api::{WeaponFiringPolicy, WeaponFiringResourceContext};

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Configuration {
    apply_damage: BTreeMap<String, bool>,
}

impl Configuration {
    pub(crate) fn configure<H: Host>(&mut self, host: &H) -> Result<(), String> {
        self.apply_damage.clear();
        // The final source retains the ordered physical reads. Use its native
        // Nullable<bool> reader: invalid/empty later values do not erase a
        // successful earlier value, and an absent override remains unset.
        // A raw latest-value presence probe cannot replace that history.
        let view = ConfigView::native_only(host);
        for name in host.registry("$weapons") {
            let section = view.section_or_empty(&name);
            if let Some(value) = section.read_ares_nullable_bool("ApplyDamage", None) {
                self.apply_damage.insert(name, value);
            }
        }
        view.finish()
    }

    pub(crate) fn policy(&self, context: &WeaponFiringResourceContext) -> WeaponFiringPolicy {
        // Ares Fire_AllowDamage1004A370, installed6FE31C: a present
        // Nullable<bool> overrides the Sonic/fire-particle default. This does
        // not select the separate EstimateDamage6FDB80 path.
        let apply_damage = self
            .apply_damage
            .get(&context.weapon_name)
            .copied()
            .unwrap_or(!context.is_sonic && !context.use_fire_particles);
        // Ares TechnoClass_Fire_Facings10047CF0, installed6FF2D1: floor(log2(Count)),
        // fewer than eight entries select the first; exponents >=16 use the
        // direction-word special case. The host owns its actual index/read.
        let muzzle_facings = match context.animation_count {
            0 => 0,
            count => {
                let exponent = count.ilog2();
                if exponent < 3 {
                    1
                } else {
                    1 << exponent.min(16)
                }
            }
        };
        WeaponFiringPolicy {
            apply_damage,
            muzzle_facings,
        }
    }
}
