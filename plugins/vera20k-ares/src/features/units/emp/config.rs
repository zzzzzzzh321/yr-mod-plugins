//! Installed Ares Type suffix716123/716132->10040DF0, WH suffix75DEA0,
//! and BeforeTypeData679A15->1002AB60/1002AECF. Host readers own numeric,
//! mission and registered-pointer parsing; this module holds no live status.
use crate::support::{ConfigView, Host};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use vera_mod_api::{
    EmpResourceContext, EmpResourcePolicy, EmpTypePolicy, EmpWarheadPolicy, ProductionReadSite,
    ProductionRegistryView, WEAPON_USER_AIRCRAFT, WEAPON_USER_BUILDING, WEAPON_USER_INFANTRY,
    WEAPON_USER_UNIT,
};

#[derive(Default, Serialize, Deserialize)]
struct Globals {
    ai_recover_mission: Option<u16>,
}

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Configuration {
    globals: Globals,
    types: BTreeMap<u8, BTreeMap<String, EmpTypePolicy>>,
    warheads: BTreeMap<String, EmpWarheadPolicy>,
}

impl Configuration {
    pub(crate) fn reset_types(&mut self) {
        // TypeReset retains RulesExt+160/+164's named recovery mission; only
        // ScenarioClear reconstructs that independent process-global field.
        self.types.clear();
        self.warheads.clear();
    }

    pub(crate) fn configure<H: Host>(
        &mut self,
        view: &ConfigView<'_, H>,
        physical: &ProductionRegistryView,
    ) {
        for read in &physical.reads {
            if matches!(read.site, ProductionReadSite::BeforeTypeData) {
                // Unlike production ProcessEntry controls,1002AB60 is not
                // gated by the RulesExt phase. Named lookup stays host-owned.
                self.globals.ai_recover_mission = view
                    .section_or_empty("CombatDamage")
                    .read_ares_nullable_mission(
                        "EMPAIRecoverMission",
                        self.globals.ai_recover_mission,
                    );
            }
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
                // EmpTypeRules::read_pass: retain the exact ordered fields and
                // class-specific Anim prefix. Unknown never becomes Clear.
                policy.immune = section.read_ares_nullable_bool("ImmuneToEMP", policy.immune);
                policy.modifier_bits =
                    section.read_ares_double_bits("EMP.Modifier", policy.modifier_bits);
                policy.sparkles = section.read_ares_nullable_registered_name(
                    "EMP.Sparkles",
                    policy.sparkles,
                    "$animations",
                );
                policy.threshold =
                    section.read_ares_emp_threshold("EMP.Threshold", policy.threshold);
                if let Some(tokens) = section.read_comma_list("VeteranAbilities", 2048) {
                    policy.veteran_immune = tokens
                        .iter()
                        .any(|name| name.eq_ignore_ascii_case("EMPIMMUNE"));
                }
                if let Some(tokens) = section.read_comma_list("EliteAbilities", 2048) {
                    policy.elite_immune = tokens
                        .iter()
                        .any(|name| name.eq_ignore_ascii_case("EMPIMMUNE"));
                }
                if policy != EmpTypePolicy::default() {
                    self.types.entry(user).or_default().insert(name, policy);
                }
            }
        }
        self.types.retain(|_, types| !types.is_empty());
        // Only actual WH physical-section suffixes carry these reads. The
        // armor ledger also visits missing sections and is intentionally not
        // used for EMP pointer/scalar configuration.
        for name in view.registered_names("$warheads") {
            let section = view.section_in_registry(&name, "$warheads");
            let mut policy = self.warheads.remove(&name).unwrap_or_default();
            policy.duration = section.read_ares_int("EMP.Duration", policy.duration);
            policy.cap = section.read_ares_int("EMP.Cap", policy.cap);
            policy.sparkles =
                section.read_ares_registered_name("EMP.Sparkles", policy.sparkles, "$animations");
            policy.affects_enemies =
                section.read_ares_bool("AffectsEnemies", policy.affects_enemies);
            policy.affects_owner =
                section.read_ares_nullable_bool("AffectsOwner", policy.affects_owner);
            if policy != EmpWarheadPolicy::default() {
                self.warheads.insert(name, policy);
            }
        }
    }

    pub(crate) fn query(&self, context: &EmpResourceContext, enabled: bool) -> EmpResourcePolicy {
        match context {
            EmpResourceContext::Rules => EmpResourcePolicy::Rules {
                enabled,
                ai_recover_mission: self.globals.ai_recover_mission,
                reactivation_requires_powered_center: enabled,
            },
            // Publish frozen constructor defaults even while disabled so old
            // builtin/current INI settings cannot leak through on restore.
            EmpResourceContext::Type { user, name } => EmpResourcePolicy::Type {
                policy: self
                    .types
                    .get(user)
                    .and_then(|types| types.get(name))
                    .cloned()
                    .unwrap_or_default(),
            },
            EmpResourceContext::Warhead { name } => EmpResourcePolicy::Warhead {
                policy: self.warheads.get(name).cloned().unwrap_or_default(),
            },
        }
    }
}
