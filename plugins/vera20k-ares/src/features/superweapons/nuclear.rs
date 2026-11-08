//! Nuclear's retained descriptor. Reference allocation and lookup stay in the
//! original host traversal; this module owns only immutable settings.
use crate::support::{ConfigView, Host};
use serde::{Deserialize, Serialize};
use vera_mod_api::{self as api, ConfigureSuperReadContext, NullableResourceName};

#[derive(Serialize, Deserialize)]
pub(crate) struct Nuclear {
    damage: i32,
    silo_launch: bool,
    take_off: NullableResourceName,
    psi_warning: NullableResourceName,
}

impl Nuclear {
    pub(super) fn initialize<H: Host>(
        context: &ConfigureSuperReadContext,
        owner: &str,
        view: &ConfigView<'_, H>,
    ) -> Result<Self, String> {
        Ok(Self {
            //100732F0: unset and every negative value use payload Damage.
            damage: -1,
            // Initialize1007338A; TakeOff remains nullable/unset and falls
            // back to General.NukeTakeOff at the existing runtime owner.
            silo_launch: true,
            take_off: NullableResourceName::Inherit,
            //1007337A uses FindOnly, not a factory. A PSIWARN introduced in
            // a later physical pass must not become a delayed default.
            psi_warning: view.superweapon_read(
                context.read,
                owner,
                "",
                "super-nuclear-psi-default",
                (),
            )?,
        })
    }

    pub(super) fn read<H: Host>(
        &mut self,
        context: &ConfigureSuperReadContext,
        owner: &str,
        selected: bool,
        view: &ConfigView<'_, H>,
    ) -> Result<(), String> {
        let raw = view.section_or_empty(owner);
        // Common LoadFromINI10033706..17 runs even while another custom
        // handler is selected. Invalid/missing values retain the nullable int.
        self.damage = raw.read_ares_int("SW.Damage", self.damage);
        if selected {
            // Nuclear INI10073446..83: the nullable TakeOff, ordinary
            // PsiWarning pointer and bool are physical, lookup-only reads.
            self.take_off = view.superweapon_read(
                context.read,
                owner,
                "Nuke.TakeOff",
                "super-animation-reference",
                self.take_off.clone(),
            )?;
            self.psi_warning = view.superweapon_read(
                context.read,
                owner,
                "Nuke.PsiWarning",
                "super-animation-reference",
                self.psi_warning.clone(),
            )?;
            self.silo_launch = raw.read_ares_bool("Nuke.SiloLaunch", self.silo_launch);
        }
        Ok(())
    }

    pub(super) fn policy(
        &self,
        activation_sound: NullableResourceName,
    ) -> api::SuperWeaponPayloadPolicy {
        api::SuperWeaponPayloadPolicy::Nuclear {
            damage: self.damage,
            silo_launch: self.silo_launch,
            take_off: self.take_off.clone(),
            psi_warning: self.psi_warning.clone(),
            // The common nullable reader preserves unset separately from
            // explicit none, so the host can supply General.DigSound.
            activation_sound,
        }
    }
}
