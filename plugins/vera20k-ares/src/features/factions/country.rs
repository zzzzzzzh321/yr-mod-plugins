//! CountryExt1001C530: one shared phase/parent attempt covers gameplay and
//! presentation. Primary1001C770 precedes the parent copy1001BDC0.
use super::{defaults, fixed_name};
use crate::support::{ConfigView, Host};
use serde::{Deserialize, Serialize};
use vera_mod_api::FactionCountryPolicy;

#[derive(Serialize, Deserialize)]
pub(super) struct State {
    phase: i32,
    pub(super) parent_attempted: bool,
    pub(super) policy: FactionCountryPolicy,
}
impl State {
    pub(super) fn new(id: &str) -> Self {
        Self {
            phase: 0,
            parent_attempted: false,
            policy: defaults::country(id),
        }
    }
    pub(super) fn can_read(&self) -> bool {
        (0..=2).contains(&self.phase)
    }
    pub(super) fn initialize_color<H: Host>(
        &mut self,
        read: u32,
        side: i32,
        view: &ConfigView<'_, H>,
    ) {
        if self.phase != 0 {
            return;
        }
        let color = match side {
            0 => Some(view.faction_find::<i32>(read, "AlliedLoad", "color")),
            1 => Some(view.faction_find::<i32>(read, "SovietLoad", "color")),
            2 => Some(
                view.faction_find::<i32>(read, "YuriLoad", "color")
                    .or_else(|| view.faction_find(read, "Purple", "color")),
            ),
            _ => None,
        };
        if let Some(color) = color {
            self.policy.loading.color_index = color.unwrap_or(-1);
        }
        self.phase = 1;
    }
    pub(super) fn read_primary<H: Host>(&mut self, read: u32, id: &str, view: &ConfigView<'_, H>) {
        let l = &mut self.policy.loading;
        for (key, target, file) in [
            ("File.LoadScreen", &mut l.file, true),
            ("File.LoadScreenPAL", &mut l.palette, true),
            ("LoadScreenText.Name", &mut l.name, false),
            ("LoadScreenText.SpecialName", &mut l.special, false),
            ("LoadScreenText.Brief", &mut l.brief, false),
        ] {
            let mut value = Some(std::mem::take(target));
            fixed_name(view, id, key, &mut value, file);
            *target = value.unwrap_or_default();
        }
        let p = &mut self.policy.presentation;
        p.flag = view.faction_read(read, id, "File.Flag", "faction-pcx", p.flag.take());
        for (key, target) in [
            ("File.ObserverFlag", &mut p.observer_flag),
            ("File.ObserverBackground", &mut p.observer_background),
            ("File.Taunt", &mut p.taunt),
        ] {
            fixed_name(view, id, key, target, true);
        }
        fixed_name(view, id, "MenuText.Status", &mut p.status, false);
        p.observer_flag_alt_palette = view
            .section_or_empty(id)
            .read_ares_bool("File.ObserverFlagAltPalette", p.observer_flag_alt_palette);
    }
    pub(super) fn inherit(&mut self, parent: &FactionCountryPolicy) {
        let p = &mut self.policy;
        p.veteran_buildings = parent.veteran_buildings.clone();
        p.loading = parent.loading.clone();
        p.paradrop = parent.paradrop.clone();
        //1001BDC0 does not copy the nullable AI weight.
        let ai_weight = p.presentation.ai_random_weight;
        p.presentation = parent.presentation.clone();
        p.presentation.ai_random_weight = ai_weight;
        p.presentation.list_index = p.presentation.list_index.wrapping_add(1);
    }
    pub(super) fn read_common<H: Host>(&mut self, read: u32, id: &str, view: &ConfigView<'_, H>) {
        let p = &mut self.policy;
        p.paradrop.parachute_anim = view.faction_read(
            read,
            id,
            "Parachute.Anim",
            "faction-valueable-ref:animation",
            p.paradrop.parachute_anim,
        );
        p.paradrop.aircraft = view.faction_read(
            read,
            id,
            "ParaDrop.Aircraft",
            "faction-valueable-ref:aircraft",
            p.paradrop.aircraft,
        );
        p.paradrop.types = view.faction_read(
            read,
            id,
            "ParaDrop.Types",
            "faction-valueable-vector:paradroppables",
            std::mem::take(&mut p.paradrop.types),
        );
        p.paradrop.counts = view.faction_read(
            read,
            id,
            "ParaDrop.Num",
            "faction-valueable-ints",
            std::mem::take(&mut p.paradrop.counts),
        );
        p.loading.color_index = view.faction_read(
            read,
            id,
            "LoadScreenText.Color",
            "faction-color-index",
            p.loading.color_index,
        );
        let section = view.section_or_empty(id);
        p.presentation.random_weight =
            section.read_ares_int("RandomSelectionWeight", p.presentation.random_weight);
        p.presentation.ai_random_weight = section
            .read_ares_nullable_int("AIRandomSelectionWeight", p.presentation.ai_random_weight);
        p.presentation.list_index = section.read_ares_int("ListIndex", p.presentation.list_index);
        p.veteran_buildings = view.faction_read(
            read,
            id,
            "VeteranBuildings",
            "faction-valueable-vector:buildings",
            std::mem::take(&mut p.veteran_buildings),
        );
        self.phase = 2;
    }
}
