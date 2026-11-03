//! One component supplies pure startup recipes and frozen INI gameplay settings.
//! Each feature owns its configuration/decisions; shared host imports live in support.
pub(crate) mod buildings;
pub(crate) mod defense;
pub(crate) mod factions;
pub(crate) mod houses;
pub(crate) mod presentation;
pub(crate) mod production;
pub(crate) mod resources;
pub(crate) mod units;
pub(crate) mod weapons;
