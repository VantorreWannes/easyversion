//! Core library for `easyversion`, a purely reductionist version control system.
//!
//! This library applies the principles of structural isomorphism and contextual agnosticism
//! to manage directory states. Versions and snapshots are defined purely by the mathematical
//! identity of their constituent data, decoupled from extrinsic relationships.

/// Mathematical domain models establishing the core primitives of structural identity and state.
pub mod model;

/// Functional pipelines and state transformations applied over the core domain primitives.
pub mod operations;

/// Atomic, content-addressed storage mechanisms.
pub mod store;

/// The canonical application identifier used for system-level configurations and data paths.
pub const APPLICATION: &str = "easyversion";

/// The organizational namespace used to isolate application data within the broader system environment.
pub const ORGANIZATION: &str = "wannesvantorre";

/// An optional context qualifier for the application environment, defaulting to an empty context.
pub const QUALIFIER: &str = "";
