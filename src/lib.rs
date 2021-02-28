//! Traits to implement Event Driven Architectures.

use std::error::Error;

/// Handles `Causes` ultimately producing `Effects`.
///
/// Implemented for `Aggregates` in `Event Sourcing`.
pub trait Actor<C: Cause, E: Effect, Err: Error> {
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
    use super::*;

    use simple_error::SimpleError;

    type Id = String;
    type Version = String;
    type Key = String;

    struct Command {
        actor_id: Id,
        actor_version: Version,
    }
    impl Cause for Command {
        type ActorId = Id;
        type ActorVersion = Version;
        fn actor_id(&self) -> Self::ActorId { self.actor_id.clone() }
        fn actor_version(&self) -> Self::ActorVersion { self.actor_version.clone() }
    }

    struct Event {
        version: Version,
        key: Key,
    }
    impl Effect for Event {
        type Version = Version;
        type Key = Key;
        fn version(&self) -> Self::Version { self.version.clone() }
        fn key(&self) -> Self::Key { self.key.clone() }
    }

    #[test]
    fn actor_handles_cause_returning_effect() {
        let command = Command {
            actor_id: String::from("one"),
            actor_version: String::from("two")
        };
        #[allow(dead_code)]
        struct Aggregate {
            id: Id,
            version: Version
        }
        impl Actor<Command, Event, SimpleError> for Aggregate {
            type Id = Id;
            type Version = Version;
            fn handle(&self, command: Command) -> Result<Vec<Event>, SimpleError> {
                if command.actor_id() == String::from("one") {
                    return Ok(vec![
                        Event {
                            version: String::from("1.0.0"),
                            key: String::from("alpha-1234")
                        }
                    ]);
                }
                Err(SimpleError::new("should have actor id one"))
            }
            fn apply(&mut self, _effects: Vec<Event>) -> Result<(), SimpleError> {
                Err(SimpleError::new("shouldn't be called"))
            }
        }
        let aggregate = Aggregate {
            id: String::from("alpha"),
            version: String::from("one")
        };
        let events = aggregate.handle(command);

        assert!(events.is_ok());
        assert_eq!(events.unwrap().len(), 1);
    }

    #[test]
    fn actor_handles_cause_returning_error() {
        let command = Command {
            actor_id: String::from("one"),
            actor_version: String::from("two")
        };
        #[allow(dead_code)]
        struct Aggregate {
            id: Id,
            version: Version
        }
        impl Actor<Command, Event, SimpleError> for Aggregate {
            type Id = Id;
            type Version = Version;
            fn handle(&self, _command: Command) -> Result<Vec<Event>, SimpleError> {
                Err(SimpleError::new("should have actor id one"))
            }
            fn apply(&mut self, _effects: Vec<Event>) -> Result<(), SimpleError> {
                Err(SimpleError::new("shouldn't be called"))
            }
        }
        let aggregate = Aggregate {
            id: String::from("alpha"),
            version: String::from("one")
        };
        let events = aggregate.handle(command);

        assert!(events.is_err());
    }

    #[test]
    fn actor_apply_effect_returning_ok() {
        let event = Event {
            version: String::from("1.0.0"),
            key: String::from("alpha-1234")
        };
        #[allow(dead_code)]
        struct Aggregate {
            id: Id,
            version: Version
        }
        impl Actor<Command, Event, SimpleError> for Aggregate {
            type Id = Id;
            type Version = Version;
            fn handle(&self, _command: Command) -> Result<Vec<Event>, SimpleError> {
                Err(SimpleError::new("shouldn't be called"))
            }
            fn apply(&mut self, effects: Vec<Event>) -> Result<(), SimpleError> {
                if effects.len() == 1 {
                    return Ok(());
                }
                Err(SimpleError::new("should have single effect"))
            }
        }
        let mut aggregate = Aggregate {
            id: String::from("alpha"),
            version: String::from("one")
        };
        let result = aggregate.apply(vec![event]);

        assert!(result.is_ok());
    }

    #[test]
    fn actor_apply_effect_returning_error() {
        #[allow(dead_code)]
        struct Aggregate {
            id: Id,
            version: Version
        }
        impl Actor<Command, Event, SimpleError> for Aggregate {
            type Id = Id;
            type Version = Version;
            fn handle(&self, _command: Command) -> Result<Vec<Event>, SimpleError> {
                Err(SimpleError::new("shouldn't be called"))
            }
            fn apply(&mut self, effects: Vec<Event>) -> Result<(), SimpleError> {
                if effects.len() == 1 {
                    return Ok(());
                }
                Err(SimpleError::new("should have zero effects"))
            }
        }
        let mut aggregate = Aggregate {
            id: String::from("alpha"),
            version: String::from("one")
        };
        let result = aggregate.apply(vec![]);

        assert!(result.is_err());
    }
}
