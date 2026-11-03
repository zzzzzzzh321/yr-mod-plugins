//! Frozen faction declarations and the actual Side/Country reader frontier.
//! The guest owns extension phases and one-shot inheritance. Original Country
//! bodies, Type registries, fixed EVA and mutable Houses remain host-owned.
use crate::{ProductionObjectKind, ProductionTypeRef};
use serde::{Deserialize, Serialize};

pub const MAX_FACTION_LIST: usize = 2048;
pub const MAX_FACTION_SIDES: usize = u8::MAX as usize + 1;
pub const MAX_FACTION_COUNTRIES: usize = u16::MAX as usize + 1;

/// A nullable pointer must preserve unset separately from an authored null in
/// JSON. Option<Option<T>> encodes both as null and loses this distinction.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(
    tag = "state",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum FactionNullableRef<T> {
    Unset,
    Null,
    Resolved(T),
}
impl<T> Default for FactionNullableRef<T> {
    fn default() -> Self {
        Self::Unset
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum FactionResourceContext {
    Side { ordinal: u32, stored_id: String },
    Country { ordinal: u32, stored_id: String },
}

/// Family counts at one actual extension read. Full Type/Anim/Color catalogs
/// stay in the host sidecar; later names must not repair an earlier lookup.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FactionRegistryPrefix {
    pub sides: u32,
    pub countries: u32,
    pub infantry: u32,
    pub units: u32,
    pub aircraft: u32,
    pub buildings: u32,
    pub animations: u32,
    /// Count of retained [Colors] entries, not the doubled runtime array.
    pub colors: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum FactionReadSite {
    /// One actual Side dispatch by ReadTypeData1002EFC0, even without a section.
    SideSweep { side: u32 },
    /// Both Country body success51214F and failure51215A reach1001D110.
    /// Parent and Side are the original body's reached fields, not final INI.
    CountrySuffix {
        country: u32,
        primary_rules: bool,
        body_read: bool,
        parent: String,
        side: i32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FactionReadPoint {
    pub prefix: FactionRegistryPrefix,
    pub site: FactionReadSite,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FactionRegistryView {
    /// Ordered stored IDs, including duplicate truncated IDs.
    pub sides: Vec<String>,
    pub countries: Vec<String>,
    pub reads: Vec<FactionReadPoint>,
}

/// Every faction scalar/vector service carries the exact read ordinal. The
/// selected physical INI and immutable registry sidecar belong to the Host.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FactionReadValue<T> {
    pub read: u32,
    pub current: T,
}

/// Literal Find used by a constructor/parent attempt, without a second parser.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FactionLookup {
    pub read: u32,
    pub name: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FactionSidePresentation {
    pub campaign_score_background: Option<String>,
    pub campaign_score_transition: Option<String>,
    pub campaign_score_animation: Option<String>,
    pub campaign_score_palette: Option<String>,
    pub multiplayer_score_background: Option<String>,
    pub multiplayer_score_bars: Option<String>,
    pub multiplayer_score_palette: Option<String>,
    pub campaign_score_under_par_theme: Option<String>,
    pub campaign_score_over_par_theme: Option<String>,
    pub multiplayer_score_win_theme: Option<String>,
    pub multiplayer_score_lose_theme: Option<String>,
    pub graphical_text_image: Option<String>,
    pub graphical_text_palette: Option<String>,
    pub dialog_background_image: Option<String>,
    pub dialog_background_palette: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FactionSidePolicy {
    pub default_disguise: FactionNullableRef<ProductionTypeRef>,
    pub crew: FactionNullableRef<ProductionTypeRef>,
    pub engineer: FactionNullableRef<ProductionTypeRef>,
    pub technician: FactionNullableRef<ProductionTypeRef>,
    pub survivor_divisor: Option<i32>,
    pub base_defenses: Option<Vec<u32>>,
    pub base_defense_counts: Option<Vec<i32>>,
    pub paradrop_types: Option<Vec<ProductionTypeRef>>,
    pub paradrop_num: Option<Vec<i32>>,
    pub paradrop_aircraft: FactionNullableRef<ProductionTypeRef>,
    pub parachute_anim: FactionNullableRef<u32>,
    pub tooltip_color: [u8; 3],
    pub message_text_color_index: i32,
    pub sidebar_mix_file_index: i32,
    pub sidebar_yuri_file_names: bool,
    pub eva_voice_index: i32,
    /// Retained baseline field; no HunterSeeker launch is implied.
    pub hunter_seeker: FactionNullableRef<ProductionTypeRef>,
    pub presentation: FactionSidePresentation,
}

impl Default for FactionSidePolicy {
    fn default() -> Self {
        Self {
            default_disguise: FactionNullableRef::Unset,
            crew: FactionNullableRef::Unset,
            engineer: FactionNullableRef::Unset,
            technician: FactionNullableRef::Unset,
            survivor_divisor: None,
            base_defenses: None,
            base_defense_counts: None,
            paradrop_types: None,
            paradrop_num: None,
            paradrop_aircraft: FactionNullableRef::Unset,
            parachute_anim: FactionNullableRef::Unset,
            tooltip_color: [0; 3],
            message_text_color_index: 0,
            sidebar_mix_file_index: 0,
            sidebar_yuri_file_names: false,
            eva_voice_index: -1,
            hunter_seeker: FactionNullableRef::Unset,
            presentation: FactionSidePresentation::default(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FactionCountryParadrop {
    pub parachute_anim: Option<u32>,
    pub aircraft: Option<ProductionTypeRef>,
    pub types: Vec<ProductionTypeRef>,
    pub counts: Vec<i32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FactionCountryLoading {
    pub file: String,
    pub palette: String,
    pub name: String,
    pub special: String,
    pub brief: String,
    pub color_index: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FactionCountryPresentation {
    pub flag: Option<String>,
    pub status: Option<String>,
    pub taunt: Option<String>,
    pub observer_background: Option<String>,
    pub observer_flag: Option<String>,
    pub observer_flag_alt_palette: bool,
    pub list_index: i32,
    pub random_weight: i32,
    pub ai_random_weight: Option<i32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FactionCountryPolicy {
    pub veteran_buildings: Vec<u32>,
    pub paradrop: FactionCountryParadrop,
    pub loading: FactionCountryLoading,
    pub presentation: FactionCountryPresentation,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum FactionResourcePolicy {
    /// Some(Policy { policy: None }) explicitly selects stock getters. An
    /// absent whole resource response remains the legacy guest/profile route.
    Side { policy: Option<FactionSidePolicy> },
    Country {
        policy: Option<FactionCountryPolicy>,
    },
}

fn stored_id_valid(id: &str) -> bool {
    !id.is_empty() && id.chars().count() <= 24 && !id.chars().any(char::is_control)
}
fn fixed_string_valid(value: &str) -> bool {
    value.chars().count() <= 31 && !value.chars().any(char::is_control)
}
fn type_valid(value: &ProductionTypeRef, kinds: &[ProductionObjectKind]) -> bool {
    kinds.contains(&value.category)
}
fn nullable_type_valid(
    value: &FactionNullableRef<ProductionTypeRef>,
    kind: ProductionObjectKind,
) -> bool {
    match value {
        FactionNullableRef::Resolved(value) => type_valid(value, &[kind]),
        _ => true,
    }
}

impl FactionRegistryView {
    pub fn validate(&self) -> Result<(), String> {
        if self.sides.len() > MAX_FACTION_SIDES
            || self.countries.len() > MAX_FACTION_COUNTRIES
            || self
                .sides
                .iter()
                .chain(&self.countries)
                .any(|id| !stored_id_valid(id))
        {
            return Err("invalid faction registry catalog".into());
        }
        for point in &self.reads {
            if point.prefix.sides as usize > self.sides.len()
                || point.prefix.countries as usize > self.countries.len()
            {
                return Err("faction read prefix exceeds its catalog".into());
            }
            match &point.site {
                FactionReadSite::SideSweep { side } if *side >= point.prefix.sides => {
                    return Err("faction Side read identity is outside its prefix".into());
                }
                FactionReadSite::CountrySuffix {
                    country, parent, ..
                } if *country >= point.prefix.countries
                    || parent.chars().count() > 24
                    || parent.chars().any(char::is_control) =>
                {
                    return Err("invalid faction Country read identity/parent".into());
                }
                _ => {}
            }
        }
        // Remaining family-prefix lengths are checked against the Host's one
        // authoritative sidecar, not against a duplicate guest catalog.
        Ok(())
    }
}

impl FactionResourcePolicy {
    pub fn validate_for(&self, context: &FactionResourceContext) -> Result<(), String> {
        let (ordinal, id, limit) = match context {
            FactionResourceContext::Side { ordinal, stored_id } => {
                (*ordinal, stored_id, MAX_FACTION_SIDES)
            }
            FactionResourceContext::Country { ordinal, stored_id } => {
                (*ordinal, stored_id, MAX_FACTION_COUNTRIES)
            }
        };
        if ordinal as usize >= limit || !stored_id_valid(id) {
            return Err("invalid faction query identity".into());
        }
        use ProductionObjectKind::{Aircraft, Infantry, Unit};
        match (self, context) {
            (Self::Side { policy }, FactionResourceContext::Side { .. }) => {
                if let Some(p) = policy {
                    let texts = &p.presentation;
                    let invalid_text = [
                        &texts.campaign_score_background,
                        &texts.campaign_score_transition,
                        &texts.campaign_score_animation,
                        &texts.campaign_score_palette,
                        &texts.multiplayer_score_background,
                        &texts.multiplayer_score_bars,
                        &texts.multiplayer_score_palette,
                        &texts.campaign_score_under_par_theme,
                        &texts.campaign_score_over_par_theme,
                        &texts.multiplayer_score_win_theme,
                        &texts.multiplayer_score_lose_theme,
                        &texts.graphical_text_image,
                        &texts.graphical_text_palette,
                        &texts.dialog_background_image,
                        &texts.dialog_background_palette,
                    ]
                    .into_iter()
                    .any(|text| text.as_ref().is_some_and(|s| !fixed_string_valid(s)));
                    let lists_too_long = p
                        .base_defenses
                        .as_ref()
                        .is_some_and(|x| x.len() > MAX_FACTION_LIST)
                        || p.base_defense_counts
                            .as_ref()
                            .is_some_and(|x| x.len() > MAX_FACTION_LIST)
                        || p.paradrop_types
                            .as_ref()
                            .is_some_and(|x| x.len() > MAX_FACTION_LIST)
                        || p.paradrop_num
                            .as_ref()
                            .is_some_and(|x| x.len() > MAX_FACTION_LIST);
                    if invalid_text
                        || lists_too_long
                        || ![&p.default_disguise, &p.crew, &p.engineer, &p.technician]
                            .into_iter()
                            .all(|v| nullable_type_valid(v, Infantry))
                        || !nullable_type_valid(&p.paradrop_aircraft, Aircraft)
                        || !nullable_type_valid(&p.hunter_seeker, Unit)
                        || p.paradrop_types
                            .as_ref()
                            .is_some_and(|v| v.iter().any(|x| !type_valid(x, &[Infantry, Unit])))
                    {
                        return Err("invalid faction Side policy".into());
                    }
                }
            }
            (Self::Country { policy }, FactionResourceContext::Country { .. }) => {
                if let Some(p) = policy {
                    let l = &p.loading;
                    let v = &p.presentation;
                    if [&l.file, &l.palette, &l.name, &l.special, &l.brief]
                        .into_iter()
                        .any(|x| !fixed_string_valid(x))
                        || [
                            &v.flag,
                            &v.status,
                            &v.taunt,
                            &v.observer_background,
                            &v.observer_flag,
                        ]
                        .into_iter()
                        .any(|x| x.as_ref().is_some_and(|s| !fixed_string_valid(s)))
                        || p.veteran_buildings.len() > MAX_FACTION_LIST
                        || p.paradrop.types.len() > MAX_FACTION_LIST
                        || p.paradrop.counts.len() > MAX_FACTION_LIST
                        || p.paradrop
                            .aircraft
                            .as_ref()
                            .is_some_and(|x| !type_valid(x, &[Aircraft]))
                        || p.paradrop
                            .types
                            .iter()
                            .any(|x| !type_valid(x, &[Infantry, Unit]))
                    {
                        return Err("invalid faction Country policy".into());
                    }
                }
            }
            _ => return Err("faction policy does not match its query kind".into()),
        }
        // All referenced ordinals are checked against the accepted host catalogs
        // before binding. Counts, signed indices and unequal lists are preserved.
        Ok(())
    }
}
