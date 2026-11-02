//! Features are included in one component and selected by frozen INI settings.
//! Each feature owns its configuration/decisions; shared host imports live in support.
pub(crate) mod defense;
pub(crate) mod houses;
pub(crate) mod production;
pub(crate) mod units;
pub(crate) mod weapons;
