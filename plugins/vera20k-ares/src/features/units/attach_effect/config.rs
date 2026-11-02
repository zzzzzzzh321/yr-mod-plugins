//! Ares shared descriptor ReadINI10058A50, called for WarheadExt+88 and
//! TechnoTypeExt+290. Host readers retain native scalar and registered-name
//! semantics; this module neither parses text nor allocates registry members.
use crate::support::{ConfigView, Host, Section};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use vera_mod_api::{
    ConfigureContext, WEAPON_USER_AIRCRAFT, WEAPON_USER_BUILDING, WEAPON_USER_INFANTRY,
    WEAPON_USER_UNIT,
};

const ONE_BITS: u64 = 1.0f64.to_bits();

/// Immutable authored values. Doubles keep their original binary64 encoding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Descriptor {
    pub(crate) duration: i32,
    pub(crate) cumulative: bool,
    pub(crate) animation: Option<String>,
    pub(crate) anim_reset_on_reapply: bool,
    pub(crate) temporal_hides_anim: bool,
    pub(crate) penetrates_iron_curtain: bool,
    pub(crate) force_decloak: bool,
    pub(crate) discard_on_entry: bool,
    pub(crate) firepower_multiplier_bits: u64,
    pub(crate) armor_multiplier_bits: u64,
    pub(crate) rof_multiplier_bits: u64,
    pub(crate) speed_multiplier_bits: u64,
    pub(crate) cloakable: bool,
    pub(crate) delay: i32,
    pub(crate) initial_delay: i32,
}

impl Default for Descriptor {
    fn default() -> Self {
        // WarheadExt constructor10051951..100519AB; TechnoTypeExt
        // constructor1003CA16..1003CABC initializes the same embedded fields.
        Self {
            duration: 0,
            cumulative: false,
            animation: None,
            anim_reset_on_reapply: false,
            temporal_hides_anim: false,
            penetrates_iron_curtain: false,
            force_decloak: false,
            discard_on_entry: false,
            firepower_multiplier_bits: ONE_BITS,
            armor_multiplier_bits: ONE_BITS,
            rof_multiplier_bits: ONE_BITS,
            speed_multiplier_bits: ONE_BITS,
            cloakable: false,
            delay: 0,
            initial_delay: 0,
        }
    }
}

impl Descriptor {
    fn read_pass<H: Host>(&mut self, section: &Section<'_, H>) {
        // Exact10058A59..10058B3A order. In particular, the registered Anim
        // pointer reads on this physical pass: unknown keeps the prior value,
        // successful none clears, and a later registration is not retroactive.
        // Section readers already probe one exact key before encoding its
        // value. Do not import every unrelated retail key as owned strings.
        self.duration = section.read_ares_int("AttachEffect.Duration", self.duration);
        self.cumulative = section.read_ares_bool("AttachEffect.Cumulative", self.cumulative);
        self.animation = section.read_ares_registered_name(
            "AttachEffect.Animation",
            self.animation.clone(),
            "$animations",
        );
        self.anim_reset_on_reapply = section.read_ares_bool(
            "AttachEffect.AnimResetOnReapply",
            self.anim_reset_on_reapply,
        );
        self.temporal_hides_anim =
            section.read_ares_bool("AttachEffect.TemporalHidesAnim", self.temporal_hides_anim);
        self.penetrates_iron_curtain = section.read_ares_bool(
            "AttachEffect.PenetratesIronCurtain",
            self.penetrates_iron_curtain,
        );
        self.force_decloak =
            section.read_ares_bool("AttachEffect.ForceDecloak", self.force_decloak);
        self.discard_on_entry =
            section.read_ares_bool("AttachEffect.DiscardOnEntry", self.discard_on_entry);
        self.firepower_multiplier_bits = section.read_ares_double_bits(
            "AttachEffect.FirepowerMultiplier",
            self.firepower_multiplier_bits,
        );
        self.armor_multiplier_bits = section
            .read_ares_double_bits("AttachEffect.ArmorMultiplier", self.armor_multiplier_bits);
        self.rof_multiplier_bits =
            section.read_ares_double_bits("AttachEffect.ROFMultiplier", self.rof_multiplier_bits);
        self.speed_multiplier_bits = section
            .read_ares_double_bits("AttachEffect.SpeedMultiplier", self.speed_multiplier_bits);
        self.cloakable = section.read_ares_bool("AttachEffect.Cloakable", self.cloakable);
        self.delay = section.read_ares_int("AttachEffect.Delay", self.delay);
        self.initial_delay = section.read_ares_int("AttachEffect.InitialDelay", self.initial_delay);
    }
}

/// Warhead House admission belongs beside its descriptor, not to a same-named
/// object type. The host's existing House/alliance owner consumes these facts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct WarheadDescriptor {
    pub(crate) affects_enemies: bool,
    pub(crate) affects_owner: Option<bool>,
    pub(crate) effect: Descriptor,
}

impl Default for WarheadDescriptor {
    fn default() -> Self {
        // WarheadExt10051937..1005193F: enemies=true, owner override unset.
        Self {
            affects_enemies: true,
            affects_owner: None,
            effect: Descriptor::default(),
        }
    }
}

impl WarheadDescriptor {
    fn read_pass<H: Host>(&mut self, section: &Section<'_, H>) {
        self.affects_enemies = section.read_ares_bool("AffectsEnemies", self.affects_enemies);
        self.affects_owner = section.read_ares_nullable_bool("AffectsOwner", self.affects_owner);
        self.effect.read_pass(section);
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct Configuration {
    // Native Techno registries retain independent identities, even when a
    // BuildingType and UnitType use the same stored name.
    objects: BTreeMap<u8, BTreeMap<String, Descriptor>>,
    warheads: BTreeMap<String, WarheadDescriptor>,
}

impl Configuration {
    pub(crate) fn configure<H: Host>(
        &mut self,
        context: &ConfigureContext,
        view: &ConfigView<'_, H>,
    ) {
        if context.final_pass {
            // Only constrain the frozen directories. Replaying a projected
            // pointer read here could accept an animation registered too late.
            // Retail has no AE overrides: do not copy its entire type and
            // warhead directories just to retain empty sparse maps.
            if !self.objects.is_empty() {
                let objects = context
                    .object_types
                    .iter()
                    .map(String::as_str)
                    .collect::<BTreeSet<_>>();
                for members in self.objects.values_mut() {
                    members.retain(|name, value| {
                        objects.contains(name.as_str()) && *value != Descriptor::default()
                    });
                }
                self.objects.retain(|_, members| !members.is_empty());
            }
            if !self.warheads.is_empty() {
                let warheads = context
                    .warheads
                    .iter()
                    .map(String::as_str)
                    .collect::<BTreeSet<_>>();
                self.warheads.retain(|name, value| {
                    warheads.contains(name.as_str()) && *value != WarheadDescriptor::default()
                });
            }
            return;
        }

        // Each scoped section carries this physical suffix's registry read
        // point, including the AnimType prefix available at that moment.
        for (user, registry) in [
            (WEAPON_USER_BUILDING, "$buildings"),
            (WEAPON_USER_AIRCRAFT, "$aircraft"),
            (WEAPON_USER_UNIT, "$vehicles"),
            (WEAPON_USER_INFANTRY, "$infantry"),
        ] {
            for name in view.registered_names(registry) {
                let section = view.section_in_registry(&name, registry);
                let mut value = self
                    .objects
                    .get_mut(&user)
                    .and_then(|members| members.remove(&name))
                    .unwrap_or_default();
                value.read_pass(&section);
                if value != Descriptor::default() {
                    self.objects.entry(user).or_default().insert(name, value);
                }
            }
        }
        self.objects.retain(|_, members| !members.is_empty());
        for name in view.registered_names("$warheads") {
            let section = view.section_in_registry(&name, "$warheads");
            let mut value = self.warheads.remove(&name).unwrap_or_default();
            value.read_pass(&section);
            if value != WarheadDescriptor::default() {
                self.warheads.insert(name, value);
            }
        }
    }

    pub(crate) fn type_effect(&self, name: &str, user: u8) -> Option<&Descriptor> {
        self.objects.get(&user)?.get(name)
    }

    pub(crate) fn warhead_effect(&self, name: &str) -> Option<&WarheadDescriptor> {
        self.warheads.get(name)
    }
}
