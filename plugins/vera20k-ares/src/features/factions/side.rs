//! SideExt1002D9F0 construction,1002E190 first-read defaults and ordered
//!1002E450..1002EA4C reads. General fallback and live consumers stay in Host.
use super::fixed_name;
use crate::support::{ConfigView, Host};
use serde::{Deserialize, Serialize};
use vera_mod_api::{
    FactionNullableRef, FactionSidePolicy, ProductionObjectKind, ProductionTypeRef,
};

#[derive(Default, Serialize, Deserialize)]
pub(super) struct State {
    phase: i32,
    pub(super) policy: FactionSidePolicy,
}

impl State {
    pub(super) fn read<H: Host>(
        &mut self,
        read: u32,
        ordinal: u32,
        id: &str,
        view: &ConfigView<'_, H>,
    ) {
        if !(0..=2).contains(&self.phase) {
            return;
        }
        if self.phase == 0 {
            self.initialize(read, ordinal, id, view);
            self.phase = 1;
        }
        let section = view.section_or_empty(id);
        let p = &mut self.policy;
        p.base_defense_counts = view.faction_read(
            read,
            id,
            "AI.BaseDefenseCounts",
            "faction-nullable-ints",
            p.base_defense_counts.take(),
        );
        p.base_defenses = view.faction_read(
            read,
            id,
            "AI.BaseDefenses",
            "faction-nullable-vector:buildings",
            p.base_defenses.take(),
        );
        for (key, target) in [
            ("Crew", &mut p.crew),
            ("Engineer", &mut p.engineer),
            ("Technician", &mut p.technician),
            ("DefaultDisguise", &mut p.default_disguise),
        ] {
            *target = view.faction_read(
                read,
                id,
                key,
                "faction-nullable-ref:infantry",
                std::mem::take(target),
            );
        }
        p.eva_voice_index =
            view.faction_read(read, id, "EVA.Tag", "faction-eva-index", p.eva_voice_index);
        p.parachute_anim = view.faction_read(
            read,
            id,
            "Parachute.Anim",
            "faction-nullable-ref:animation",
            std::mem::take(&mut p.parachute_anim),
        );
        p.paradrop_aircraft = view.faction_read(
            read,
            id,
            "ParaDrop.Aircraft",
            "faction-nullable-ref:aircraft",
            std::mem::take(&mut p.paradrop_aircraft),
        );
        p.paradrop_types = view.faction_read(
            read,
            id,
            "ParaDrop.Types",
            "faction-nullable-vector:paradroppables",
            p.paradrop_types.take(),
        );
        p.paradrop_num = view.faction_read(
            read,
            id,
            "ParaDrop.Num",
            "faction-nullable-ints",
            p.paradrop_num.take(),
        );
        p.sidebar_mix_file_index =
            section.read_ares_int("Sidebar.MixFileIndex", p.sidebar_mix_file_index);
        p.sidebar_yuri_file_names =
            section.read_ares_bool("Sidebar.YuriFileNames", p.sidebar_yuri_file_names);
        p.tooltip_color = section.read_color_rgb("ToolTipColor", p.tooltip_color, None);
        p.survivor_divisor = section.read_ares_nullable_int("SurvivorDivisor", p.survivor_divisor);
        p.message_text_color_index = view.faction_read(
            read,
            id,
            "MessageTextColor",
            "faction-color-index",
            p.message_text_color_index,
        );
        p.hunter_seeker = view.faction_read(
            read,
            id,
            "HunterSeeker",
            "faction-nullable-ref:unit",
            std::mem::take(&mut p.hunter_seeker),
        );
        let s = &mut p.presentation;
        for (key, target) in [
            ("CampaignScore.Background", &mut s.campaign_score_background),
            ("CampaignScore.Transition", &mut s.campaign_score_transition),
            ("CampaignScore.Animation", &mut s.campaign_score_animation),
            ("CampaignScore.Palette", &mut s.campaign_score_palette),
            (
                "MultiplayerScore.Background",
                &mut s.multiplayer_score_background,
            ),
            ("MultiplayerScore.Bars", &mut s.multiplayer_score_bars),
            ("MultiplayerScore.Palette", &mut s.multiplayer_score_palette),
            (
                "CampaignScore.UnderParTheme",
                &mut s.campaign_score_under_par_theme,
            ),
            (
                "CampaignScore.OverParTheme",
                &mut s.campaign_score_over_par_theme,
            ),
            (
                "MultiplayerScore.WinTheme",
                &mut s.multiplayer_score_win_theme,
            ),
            (
                "MultiplayerScore.LoseTheme",
                &mut s.multiplayer_score_lose_theme,
            ),
            ("GraphicalText.Image", &mut s.graphical_text_image),
            ("GraphicalText.Palette", &mut s.graphical_text_palette),
            ("DialogBackground.Image", &mut s.dialog_background_image),
            ("DialogBackground.Palette", &mut s.dialog_background_palette),
        ] {
            fixed_name(view, id, key, target, true);
        }
        self.phase = 2;
    }

    fn initialize<H: Host>(&mut self, read: u32, ordinal: u32, id: &str, view: &ConfigView<'_, H>) {
        let nod = id.eq_ignore_ascii_case("Nod");
        let third = id.eq_ignore_ascii_case("ThirdSide");
        let p = &mut self.policy;
        p.sidebar_mix_file_index = if nod || third { 2 } else { 1 };
        p.sidebar_yuri_file_names = third;
        p.eva_voice_index = if nod {
            1
        } else if third {
            2
        } else {
            0
        };
        p.tooltip_color = if nod || third {
            [255, 255, 0]
        } else {
            [164, 210, 255]
        };
        p.message_text_color_index = if nod {
            11
        } else if third {
            25
        } else {
            21
        };
        // FindIndex uses the first initialization's actual Aircraft prefix.
        p.paradrop_aircraft = match view.faction_find::<u32>(read, "PDPLANE", "aircraft") {
            Some(ordinal) => FactionNullableRef::Resolved(ProductionTypeRef {
                category: ProductionObjectKind::Aircraft,
                ordinal,
            }),
            None => FactionNullableRef::Null,
        };
        let names = match ordinal {
            0 => [
                "ASCRBKMD.SHP",
                "ASCRTMD.SHP",
                "ASCRAMD.SHP",
                "ASCORE.PAL",
                "MPASCRNL.SHP",
                "mpascrnlbar~~.pcx",
                "MPASCRN.PAL",
            ],
            1 => [
                "SSCRBKMD.SHP",
                "SSCRTMD.SHP",
                "SSCRAMD.SHP",
                "SSCORE.PAL",
                "MPSSCRNL.SHP",
                "mpsscrnlbar~~.pcx",
                "MPSSCRN.PAL",
            ],
            _ => [
                "SYCRBKMD.SHP",
                "SYCRTMD.SHP",
                "SYCRAMD.SHP",
                "YSCORE.PAL",
                "MPYSCRNL.SHP",
                "mpyscrnlbar~~.pcx",
                "MPYSCRN.PAL",
            ],
        };
        let s = &mut p.presentation;
        s.campaign_score_background = Some(names[0].into());
        s.campaign_score_transition = Some(names[1].into());
        s.campaign_score_animation = Some(names[2].into());
        s.campaign_score_palette = Some(names[3].into());
        s.multiplayer_score_background = Some(names[4].into());
        s.multiplayer_score_bars = Some(names[5].into());
        s.multiplayer_score_palette = Some(names[6].into());
        s.campaign_score_under_par_theme = Some("SCORE".into());
        s.campaign_score_over_par_theme = Some("SCORE".into());
        s.multiplayer_score_win_theme = Some("SCORE".into());
        s.multiplayer_score_lose_theme = Some("SCORE".into());
    }
}
