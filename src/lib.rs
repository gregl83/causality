//! Traits to implement Event Driven Architectures.

/// Handles `Causes` ultimately producing `Effects`.
///
/// Implemented for `Aggregates` in `Event Sourcing`.
pub trait Actor<C: Cause, E: Effect, Err> {
    /// Unique Id for `Actor`.
    type Id;
    /// Version of `Actor` dependent on `Effects` applied.
    type Version;
    /// Handle `Cause` returning vector of `Effects` or error.
    fn handle(&self, cause: C) -> Result<Vec<E>, Err>;
    /// Apply `Effects` on Actor.
    fn apply(&mut self, effects: Vec<E>) -> Result<(), Err>;
}

/// Action that is expected to produce `Effects`.
///
/// Implemented for actions on `Actors`.
pub trait Cause {
    /// Unique `Actor` Id.
    type ActorId;
    /// Version of `Actor` handling `Cause` for ordering (optimistic concurrency).
    type ActorVersion;
    /// Returns unique `Actor` Id
    fn actor_id(&self) -> Self::ActorId;
    /// Returns `Actor` version.
    fn actor_version(&self) -> Self::ActorVersion;
}

/// Event that *can* impact `Actors`.
///
/// Implemented for events produced by `Actors` handling `Causes`.
pub trait Effect {
    /// Data structure version.
    type Version;
    /// Unique key used for idempotency (duplicate detection).
    type Key;
    /// Returns version.
    fn version(&self) -> Self::Version;
    /// Returns unique key.
    fn key(&self) -> Self::Key;
}

#[cfg(test)]
mod tests {
    // TODO !!!

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
