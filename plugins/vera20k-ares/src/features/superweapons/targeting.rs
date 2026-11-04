//! Ares targeting vocabulary/default rows. Numeric/token parsing stays in Host.
use crate::support::{Host, Section};
use serde::{Deserialize, Serialize};

pub(super) const HOUSE_TOKENS: [(&str, u32); 8] = [
    ("owner", 1),
    ("allies", 2),
    ("enemies", 4),
    ("team", 3),
    ("trump", 5),
    ("others", 6),
    ("all", 7),
    ("none", 0),
];
const TARGET_TOKENS: [(&str, u32); 8] = [
    ("land", 1),
    ("water", 2),
    ("empty", 4),
    ("infantry", 8),
    ("units", 16),
    ("buildings", 32),
    ("all", 255),
    ("none", 0),
];
const CONSTRAINT_TOKENS: [(&str, u32); 10] = [
    ("none", 0),
    ("offensive_cell_clear", 1),
    ("defensive_cell_clear", 2),
    ("enemy", 4),
    ("lightningstorm_inactive", 8),
    ("dominator_inactive", 16),
    ("attacked", 32),
    ("lowpower", 64),
    ("offensive_cell_set", 128),
    ("defensive_cell_set", 256),
];

/// Reader10034350 names1009BAF8; mask/constraint defaults table1009B8A4.
const MODES: [(&str, u8, u8, u16); 21] = [
    ("none", 0, 0, 0),
    ("nuke", 0x38, 4, 4),
    ("lightningstorm", 0x38, 4, 12),
    ("psychicdominator", 0x18, 7, 21),
    ("paradrop", 0, 0, 0),
    ("geneticmutator", 8, 7, 1),
    ("forceshield", 0, 0, 0),
    ("notarget", 0, 0, 0),
    ("offensive", 0x38, 4, 4),
    ("stealth", 0x38, 4, 0),
    ("self", 0, 0, 0),
    ("base", 0, 0, 0),
    ("multimissile", 32, 4, 4),
    ("hunterseeker", 0, 0, 4),
    ("enemybase", 0, 0, 4),
    ("ironcurtain", 0, 0, 0),
    ("attack", 0, 0, 32),
    ("lowpower", 0, 0, 64),
    ("lowpowerattack", 0, 0, 96),
    ("droppod", 0, 0, 4),
    ("lightningrandom", 0, 0, 0),
];

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Targeting {
    pub(crate) mode: u8,
    pub(crate) constraints: Option<u16>,
    pub(crate) manual_target: u8,
    pub(crate) manual_house: u8,
    pub(crate) ai_target: Option<u8>,
    pub(crate) ai_house: Option<u8>,
}

impl Targeting {
    /// LoadFromINI100335D2..1003367D. Unknown preserves, successful zero
    /// sets the nullable flag. No aliases, trimming or guest mask parser.
    pub(super) fn read<H: Host>(&mut self, section: &Section<'_, H>) -> Result<(), String> {
        if let Some(name) = section.read_name("SW.AITargeting", 2048)
            && let Some(mode) = MODES
                .iter()
                .position(|(mode, ..)| mode.eq_ignore_ascii_case(&name))
        {
            self.mode = mode as u8;
        }
        if let Some(value) =
            section.read_comma_mask("SW.AITargeting.Constraints", &CONSTRAINT_TOKENS)
        {
            self.constraints = Some(u16::try_from(value).map_err(|e| e.to_string())?);
        }
        if let Some(value) = section.read_comma_mask("SW.RequiresTarget", &TARGET_TOKENS) {
            self.manual_target = u8::try_from(value).map_err(|e| e.to_string())?;
        }
        if let Some(value) = section.read_comma_mask("SW.RequiresHouse", &HOUSE_TOKENS) {
            self.manual_house = u8::try_from(value).map_err(|e| e.to_string())?;
        }
        if let Some(value) = section.read_comma_mask("SW.AIRequiresTarget", &TARGET_TOKENS) {
            self.ai_target = Some(u8::try_from(value).map_err(|e| e.to_string())?);
        }
        if let Some(value) = section.read_comma_mask("SW.AIRequiresHouse", &HOUSE_TOKENS) {
            self.ai_house = Some(u8::try_from(value).map_err(|e| e.to_string())?);
        }
        Ok(())
    }
    pub(crate) fn effective_constraints(&self) -> Result<u16, String> {
        let row = MODES
            .get(self.mode as usize)
            .ok_or("invalid retained Super targeting mode")?;
        Ok(self.constraints.unwrap_or(row.3))
    }
    pub(crate) fn automatic(&self) -> Result<(u8, u8), String> {
        let &(_, target, house, _) = MODES
            .get(self.mode as usize)
            .ok_or("invalid retained Super targeting mode")?;
        Ok((
            self.ai_target.unwrap_or(target),
            self.ai_house.unwrap_or(house),
        ))
    }
    /// Other recognized modes remain represented but have no AI dispatch.
    pub(crate) fn offensive(&self) -> bool {
        self.mode == 8
    }
}

/// Handler Initialize vtable+14 stores SWTypeExt+210 once (10033337).
pub(super) fn handler_initial_mode(type_name: &str) -> u8 {
    const MODES: [(&str, u8); 17] = [
        ("MultiMissile", 1), //100733BC
        ("IronCurtain", 15),
        ("Protect", 15),       //10074BFE
        ("LightningStorm", 2), //10072CD5
        ("ParaDrop", 4),
        ("AmerParaDrop", 4),     //10073F48
        ("PsychicDominator", 3), //10070A01
        ("SpyPlane", 4),         //10075865
        ("GeneticConverter", 5), //100720DF
        ("ForceShield", 6),      //10074B9A
        ("PsychicReveal", 4),    //10075336
        ("SonarPulse", 9),       //100756E5
        ("UnitDelivery", 4),     //100759D4
        ("GenericWarhead", 8),   //10071864
        ("HunterSeeker", 13),    //100729E1
        ("DropPod", 19),         //10071125
        ("Battery", 17),         //1006ECA4
    ];
    MODES
        .iter()
        .find_map(|(name, mode)| name.eq_ignore_ascii_case(type_name).then_some(*mode))
        .unwrap_or(0)
}
