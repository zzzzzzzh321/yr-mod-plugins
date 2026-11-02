//! Immutable resource decisions queried once per admitted type or weapon.
//! The engine's existing resource owners retain all mutable counts and clocks.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeaponResourceContext {
    pub weapon_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypeResourceContext {
    pub type_name: String,
    pub user: u8,
}

pub const WEAPON_USER_UNIT: u8 = 1;
pub const WEAPON_USER_INFANTRY: u8 = 2;
pub const WEAPON_USER_BUILDING: u8 = 4;
pub const WEAPON_USER_AIRCRAFT: u8 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeaponResourcePolicy {
    pub cost: i32,
    pub reload_after_use: bool,
    pub users: u8,
}

impl WeaponResourcePolicy {
    pub fn applies_to(self, user: u8) -> bool {
        self.users & user != 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReloadPolicy {
    pub normal_amount: i32,
    pub empty_amount: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DepletedWeaponPolicy {
    pub weapon_index: i32,
    pub threshold: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypeResourcePolicy {
    pub users: u8,
    pub reload: Option<ReloadPolicy>,
    pub depleted_weapon: Option<DepletedWeaponPolicy>,
}

impl TypeResourcePolicy {
    pub fn applies_to(self, user: u8) -> bool {
        self.users & user != 0
    }
}
