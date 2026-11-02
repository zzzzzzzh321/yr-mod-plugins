//! Ares AttachEffect configuration and lifecycle decisions. The host owns the
//! only instance vector, animation handles and committed derived statistics.
mod config;
mod logic;

pub(crate) use config::{Configuration, Descriptor, WarheadDescriptor};
pub(crate) use logic::{evaluate, resource};
