//! Frozen type sound declarations against the process-selected SOUNDMD catalog.
//! The component owns configuration history; the host owns sound registration,
//! promotion state and the existing positional sound/EVA delivery.
use crate::{NullableResourceName, ProductionTypeRef};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const MAX_PROMOTION_SOUND_TYPES: usize = 65_536;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypeSoundResourceContext {}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PromotionSoundDeclaration {
    pub object: ProductionTypeRef,
    pub veteran: NullableResourceName,
    pub elite: NullableResourceName,
}

/// None on the containing Plan preserves the old profile. A present catalog
/// selects global promotion sounds for omitted Types, even in a legacy host.
/// Named cues require the exact process catalog identity; binding validates
/// every Type and cue before publishing any effective resource.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypeSoundResourcePolicy {
    pub source_hash: Option<u64>,
    pub promotions: Vec<PromotionSoundDeclaration>,
}

impl TypeSoundResourcePolicy {
    pub fn validate(&self) -> Result<(), String> {
        if self.promotions.len() > MAX_PROMOTION_SOUND_TYPES {
            return Err("type sound catalog exceeds its declaration limit".into());
        }
        let mut identities = BTreeSet::new();
        for entry in &self.promotions {
            if !identities.insert(entry.object) {
                return Err("duplicate promotion sound Type identity".into());
            }
            for cue in [&entry.veteran, &entry.elite] {
                if let NullableResourceName::Named(name) = cue {
                    // SOUNDMD IDs are stored in native31-byte names. Allow
                    // their Unicode representation without admitting an
                    // arbitrarily large unregistered scalar.
                    if self.source_hash.is_none()
                        || name.is_empty()
                        || name.len() > 128
                        || name.chars().any(char::is_control)
                    {
                        return Err("invalid or unbound promotion sound name".into());
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{API_VERSION, Context, Plan, ProductionObjectKind, Request};

    #[test]
    fn type_sound_catalog_keeps_inherit_silence_and_registered_identity_separate() {
        let request = Request {
            api_version: API_VERSION,
            context: Context::TypeSoundResource(TypeSoundResourceContext {}),
            config: vec![],
            state: vec![],
        };
        request.validate().unwrap();
        let mut plan = Plan::unchanged(&request);
        assert!(
            !serde_json::to_string(&plan)
                .unwrap()
                .contains("type_sound_resource")
        );
        let declaration = PromotionSoundDeclaration {
            object: ProductionTypeRef {
                category: ProductionObjectKind::Infantry,
                ordinal: 0,
            },
            veteran: NullableResourceName::Clear,
            elite: NullableResourceName::Named("none".into()),
        };
        let mut policy = TypeSoundResourcePolicy {
            source_hash: Some(41),
            promotions: vec![declaration.clone()],
        };
        plan.type_sound_resource = Some(policy.clone());
        plan.validate_for(&request).unwrap();
        assert_eq!(
            serde_json::from_slice::<Plan>(&serde_json::to_vec(&plan).unwrap()).unwrap(),
            plan
        );
        policy.source_hash = None;
        assert!(policy.validate().is_err());
        policy.source_hash = Some(41);
        policy.promotions.push(declaration);
        assert!(policy.validate().is_err());
        plan.type_sound_resource = Some(TypeSoundResourcePolicy::default());
        plan.validate_for(&request).unwrap();
        let mut foreign = request.clone();
        foreign.context = Context::MatchRules(crate::MatchRulesContext {});
        assert!(plan.validate_for(&foreign).is_err());
        plan.state.push(1);
        assert!(plan.validate_for(&request).is_err());
    }
}
