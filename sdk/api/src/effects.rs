//! Static effect resources and per-actor transactions. Instances, animation
//! handles and derived numeric values have one host owner. A guest decides
//! lifecycle changes from a read-only snapshot, without keeping another list.
use crate::{
    EntityContext, WEAPON_USER_AIRCRAFT, WEAPON_USER_BUILDING, WEAPON_USER_INFANTRY,
    WEAPON_USER_UNIT,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectResourceKind {
    Warhead,
    Unit,
    Infantry,
    Building,
    Aircraft,
}

impl EffectResourceKind {
    pub const fn user(self) -> Option<u8> {
        match self {
            Self::Warhead => None,
            Self::Unit => Some(WEAPON_USER_UNIT),
            Self::Infantry => Some(WEAPON_USER_INFANTRY),
            Self::Building => Some(WEAPON_USER_BUILDING),
            Self::Aircraft => Some(WEAPON_USER_AIRCRAFT),
        }
    }

    pub const fn for_user(user: u8) -> Option<Self> {
        match user {
            WEAPON_USER_UNIT => Some(Self::Unit),
            WEAPON_USER_INFANTRY => Some(Self::Infantry),
            WEAPON_USER_BUILDING => Some(Self::Building),
            WEAPON_USER_AIRCRAFT => Some(Self::Aircraft),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectResourceContext {
    pub kind: EffectResourceKind,
    pub name: String,
}

/// Values folded by the host's existing ordered numeric owner. Durations,
/// stacking and admission rules remain in the component's frozen configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectDescriptor {
    pub firepower_multiplier_bits: u64,
    pub armor_multiplier_bits: u64,
    pub rof_multiplier_bits: u64,
    pub speed_multiplier_bits: u64,
    pub cloakable: bool,
    pub animation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectInstanceView {
    pub resource: EffectResourceContext,
    pub remaining: i32,
    pub source_owner: Option<String>,
    pub has_animation: bool,
}

/// Host-retained control registers for an actor's self-applied effect. The
/// guest owns their transitions; they do not advance on their own in the host.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "phase", content = "counter", rename_all = "snake_case")]
pub enum EffectControl {
    Waiting(i32),
    Active(i32),
    Disabled(i32),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EffectEvent {
    Apply {
        resource: EffectResourceContext,
        source_owner: Option<String>,
        source_is_owner: bool,
        source_is_ally: bool,
        affects_allies: bool,
        verses_bits: u64,
    },
    Visit,
    EnterLimbo,
    TypeChanged {
        previous: EffectResourceContext,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectContext {
    pub frame: i32,
    pub entity: EntityContext,
    pub user: u8,
    pub inside_transport: bool,
    pub cloaked_or_cloaking: bool,
    pub entries: Vec<EffectInstanceView>,
    pub control: EffectControl,
    pub animations_hidden: bool,
    pub event: EffectEvent,
}

/// Indices address the live list after preceding actions in this transaction.
/// Resource qualification and source-house identities are validated by the
/// host before any mutation. No action assigns raw animation handles or stats.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EffectAction {
    SetControl {
        control: EffectControl,
    },
    SetAnimationsHidden {
        hidden: bool,
    },
    SetRemaining {
        index: usize,
        remaining: i32,
    },
    Append {
        resource: EffectResourceContext,
        remaining: i32,
        source_owner: Option<String>,
    },
    /// Publish erasure, then retire any animation still held by this entry.
    Remove {
        index: usize,
    },
    CreateAnimation {
        index: usize,
    },
    RetireAnimation {
        index: usize,
    },
    RecalculateStats,
    Decloak,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectPlan {
    /// Postconditions, checked against the ordered register-write actions.
    /// Hosts commit only the actions, so animation callbacks observe the same
    /// intermediate registers as the original lifecycle owner.
    pub control: EffectControl,
    pub animations_hidden: bool,
    pub actions: Vec<EffectAction>,
}

impl EffectPlan {
    pub fn unchanged(context: &EffectContext) -> Self {
        Self {
            control: context.control,
            animations_hidden: context.animations_hidden,
            actions: Vec::new(),
        }
    }

    pub fn validate(&self, context: &EffectContext) -> Result<(), String> {
        let mut len = context.entries.len();
        let mut control = context.control;
        let mut hidden = context.animations_hidden;
        for action in &self.actions {
            match action {
                EffectAction::SetControl { control: value } => control = *value,
                EffectAction::SetAnimationsHidden { hidden: value } => hidden = *value,
                EffectAction::Append { .. } => {
                    len = len.checked_add(1).ok_or("effect instance count overflow")?;
                }
                EffectAction::SetRemaining { index, .. }
                | EffectAction::CreateAnimation { index }
                | EffectAction::RetireAnimation { index }
                | EffectAction::Remove { index } => {
                    if *index >= len {
                        return Err("effect action index is out of range".into());
                    }
                    if matches!(action, EffectAction::Remove { .. }) {
                        len -= 1;
                    }
                }
                EffectAction::RecalculateStats | EffectAction::Decloak => {}
            }
        }
        if control != self.control || hidden != self.animations_hidden {
            return Err("effect register postconditions disagree with ordered actions".into());
        }
        Ok(())
    }
}
