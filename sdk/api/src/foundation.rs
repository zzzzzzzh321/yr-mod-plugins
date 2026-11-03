//! Static geometry selection; the host owns parsing, spatial consumers and
//! saved type facts. Complete cell/outline arrays never cross the guest ABI.
use crate::{ImmutableIniSectionRef, ProductionObjectKind, ProductionTypeRef};
use serde::{Deserialize, Serialize};

pub const STOCK_FOUNDATION_COUNT: u8 = 22;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FoundationReadPoint {
    pub building: ProductionTypeRef,
    pub type_name: String,
    pub art: ImmutableIniSectionRef,
}

/// Independent ART-domain metadata. Existing production-view messages retain
/// their original shape for older components with closed deserializers.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FoundationRegistryView {
    pub reads: Vec<FoundationReadPoint>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FoundationResourceContext {
    pub building: ProductionTypeRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum FoundationResourcePolicy {
    Stock { selector: u8 },
    ArtSection { reference: ImmutableIniSectionRef },
}

impl FoundationResourcePolicy {
    pub fn validate_for(&self, context: &FoundationResourceContext) -> Result<(), String> {
        if context.building.category != ProductionObjectKind::Building {
            return Err("foundation resources require a Building identity".into());
        }
        match self {
            Self::Stock { selector } if *selector >= STOCK_FOUNDATION_COUNT => {
                Err("unknown stock foundation selector".into())
            }
            Self::ArtSection { reference } => reference.validate(),
            _ => Ok(()),
        }
    }
}
