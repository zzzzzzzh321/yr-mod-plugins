//! Immutable armor identities and interactions, bound before constructors.
//! Readers and damage arithmetic stay host-owned; the guest defines the
//! catalog, physical-layer bindings and each warhead's ordered records.
use serde::{Deserialize, Serialize};

pub const BASE_ARMOR_NAMES: [&str; 11] = [
    "none",
    "flak",
    "plate",
    "light",
    "medium",
    "heavy",
    "wood",
    "steel",
    "concrete",
    "special_1",
    "special_2",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArmorObjectKind {
    Unit,
    Infantry,
    Building,
    Aircraft,
    Terrain,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ArmorResourceContext {
    Catalog,
    Object {
        category: ArmorObjectKind,
        name: String,
    },
    Warhead {
        name: String,
    },
}

/// Binary64 values retain their native encodings, including unordered values.
/// The flags are independent; force_fire permits an otherwise zero interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArmorVersePolicy {
    pub multiplier_bits: u64,
    pub force_fire: bool,
    pub retaliate: bool,
    pub passive_acquire: bool,
}

impl Default for ArmorVersePolicy {
    fn default() -> Self {
        Self {
            multiplier_bits: 1.0f64.to_bits(),
            force_fire: true,
            retaliate: true,
            passive_acquire: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ArmorResourcePolicy {
    Catalog {
        names: Vec<String>,
    },
    Object {
        armor: u32,
    },
    Warhead {
        entries: Vec<ArmorVersePolicy>,
        use_medium_wood_target_mask: bool,
    },
}

impl ArmorResourcePolicy {
    pub(crate) fn validate_for(&self, context: &ArmorResourceContext) -> Result<(), String> {
        match (self, context) {
            (Self::Catalog { names }, ArmorResourceContext::Catalog) => {
                if names.len() < BASE_ARMOR_NAMES.len()
                    || names.len() > u32::MAX as usize
                    || !names.iter().zip(BASE_ARMOR_NAMES).all(|(a, b)| a == b)
                    || names.iter().any(|name| name.chars().count() >= 32)
                {
                    return Err("invalid armor catalog".into());
                }
                // Native truncated identities may collide. Preserve their
                // ordered slots instead of imposing new uniqueness rules.
                Ok(())
            }
            (Self::Object { .. }, ArmorResourceContext::Object { .. }) => Ok(()),
            (Self::Warhead { entries, .. }, ArmorResourceContext::Warhead { .. })
                if entries.len() >= BASE_ARMOR_NAMES.len()
                    && entries.len() <= u32::MAX as usize =>
            {
                Ok(())
            }
            _ => Err("armor resource result does not match its query".into()),
        }
        // Cross-resource indices and lengths are validated atomically by
        // the host against this component's admitted catalog.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{API_VERSION, Context, Plan, Request};

    #[test]
    fn armor_queries_keep_the_base_domain_and_cannot_write_entity_state() {
        let request = Request {
            api_version: API_VERSION,
            context: Context::ArmorResource(ArmorResourceContext::Catalog),
            config: Vec::new(),
            state: Vec::new(),
        };
        let mut plan = Plan::unchanged(&request);
        plan.armor_resource = Some(ArmorResourcePolicy::Catalog {
            names: BASE_ARMOR_NAMES.into_iter().map(str::to_owned).collect(),
        });
        assert!(plan.validate_for(&request).is_ok());
        plan.state.push(1);
        assert!(plan.validate_for(&request).is_err());
        plan.state.clear();
        let Some(ArmorResourcePolicy::Catalog { names }) = &mut plan.armor_resource else {
            unreachable!()
        };
        names[0] = "other".into();
        assert!(plan.validate_for(&request).is_err());
        plan.armor_resource = Some(ArmorResourcePolicy::Object { armor: 0 });
        assert!(plan.validate_for(&request).is_err());
    }
}
