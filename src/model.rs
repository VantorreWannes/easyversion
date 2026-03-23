use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

/// A deterministic, mathematically derived identity for an intrinsic state.
/// This structure enforces the Axiom of Derived Identity: two entities with the same
/// structural data will yield the exact same `Id`, rendering them equivalent.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Serialize, Deserialize)]
pub struct Id {
    /// The structural hash digest representing the underlying data.
    pub digest: u64,
}

/// A purely structural map associating logical file paths with their intrinsic identities.
/// This acts as a passive registry of state, agnostic to the environment or storage mechanisms.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// The mathematical mapping from a relative path to the derived identity of its content.
    pub files: HashMap<PathBuf, Id>,
}

/// An immutable point-in-time capture of a directory's state.
/// Encapsulates a structural `Manifest` alongside an optional semantic label.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// An optional, arbitrary user-defined string describing the snapshot context.
    pub comment: Option<String>,
    /// The pure structural state captured by this snapshot.
    pub manifest: Manifest,
}

/// A sequential record of state transitions over time.
/// Provides a temporal axis to an otherwise stateless collection of snapshots.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Default)]
pub struct History {
    /// The chronologically ordered series of state captures.
    pub snapshots: Vec<Snapshot>,
}
