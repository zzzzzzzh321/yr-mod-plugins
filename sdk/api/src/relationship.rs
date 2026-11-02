//! Relationship flags shared by rule-driven presentation and Mod projections.
//!
//! The retained Phobos spelling follows `AffectedHouse` (`Utilities/Enum.h`)
//! and its `Valueable::Read` token policy (`Utilities/TemplateDef.h`). The flags
//! express ownership relationships; no shield state or behavior lives here.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct AffectedHouse(u8);

impl AffectedHouse {
    pub const NONE: Self = Self(0);
    pub const OWNER: Self = Self(1);
    pub const ALLIES: Self = Self(2);
    pub const ENEMIES: Self = Self(4);
    pub const NEUTRAL: Self = Self(8);
    pub const ALL: Self = Self(15);

    pub fn bits(self) -> u8 {
        self.0
    }

    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    /// One unknown token fails the whole read, retaining the caller's previous value.
    pub fn parse(raw: &str) -> Option<Self> {
        let mut parsed = Self::NONE;
        for part in raw.split(',') {
            let token = part.trim_matches(|c| matches!(c, ' ' | '\t' | '\r'));
            let flag = match token.to_ascii_lowercase().as_str() {
                "owner" | "self" => Self::OWNER,
                "allies" | "ally" => Self::ALLIES,
                "enemies" | "enemy" => Self::ENEMIES,
                "neutral" => Self::NEUTRAL,
                "team" => Self(Self::OWNER.0 | Self::ALLIES.0),
                "others" => Self(Self::ALLIES.0 | Self::ENEMIES.0),
                "all" => Self::ALL,
                "none" => Self::NONE,
                _ => return None,
            };
            parsed = Self(parsed.0 | flag.0);
        }
        Some(parsed)
    }
}

impl Default for AffectedHouse {
    fn default() -> Self {
        Self::ALL
    }
}

/// Ares WarheadExt10051B70's ownership precedence, shared with the legacy
/// engine adapter. Alliance facts come from the host's existing House owner.
pub fn permits_house(
    source_exists: bool,
    source_is_owner: bool,
    source_is_ally: bool,
    affects_allies: bool,
    affects_enemies: bool,
    affects_owner: Option<bool>,
) -> bool {
    !source_exists
        || if source_is_owner && affects_owner.is_some() {
            affects_owner.expect("present owner override")
        } else if source_is_ally {
            affects_allies
        } else {
            affects_enemies
        }
}
