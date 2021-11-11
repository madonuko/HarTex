//! # The `fw_setup` Module
//!
//! Utility function for setting up the command framework.

use hartex_cmdsys::framework::CommandFramework;
use hartex_core::logging::tracing;
use hartex_eventsys::{
    emitter::EventEmitter,
    events::Events
};

/// # Function `framework_setup`
///
/// Sets up the command framework.
#[must_use]
pub fn framework_setup() -> (EventEmitter, Events) {
    tracing::trace!("setting up command framework");

    let framework = CommandFramework::default();

    let listeners = framework.clone().listeners();

    (EventEmitter::new(listeners), framework.events())
}