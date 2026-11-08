//! Frozen weapon travel budgets. ProjectileStore owns the live counter and flight.
use crate::support::{ConfigView, Host};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// Ares WeaponTypeExt constructor100549AA initializes exactly 100000 leptons.
const DEFAULT_RANGE: i32 = 100_000;

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Configuration {
    ranges: BTreeMap<String, i32>,
}

impl Configuration {
    pub(crate) fn configure<H: Host>(&mut self, host: &H) -> Result<(), String> {
        self.ranges.clear();
        // Ares reader10055872 ->1000A5D0 uses the retained Weapon body, not a
        // later same-ID Projectile body. The canonical host reader owns cells
        // to signed-lepton conversion and retention after invalid/empty input.
        let view = ConfigView::native_only(host);
        for name in host.registry("$weapon-config") {
            let value = view.retained_weapon_read::<i32>(
                &name,
                "ProjectileRange",
                "ares-leptons",
                DEFAULT_RANGE,
            )?;
            if value != DEFAULT_RANGE {
                self.ranges.insert(name, value);
            }
        }
        view.finish()
    }

    pub(crate) fn initial_range(&self, weapon_name: &str) -> i32 {
        self.ranges
            .get(weapon_name)
            .copied()
            .unwrap_or(DEFAULT_RANGE)
    }
}
