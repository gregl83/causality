//! Traits for implementing `Event Driven Architectures`.
//!
//! Borrowing with fervor from the `Theory of Causality` to conceptualize deterministic state.
//!
//! Ideas represented are often a reflection off of the work of others (`Causality`, `Event Sourcing`, `CQRS`, etc).
//!
//! **CAUTION:** Implementation hasn't had time to mature. Expect breaking changes.
//!
//! # Example
//!
//! ```
//! use simple_error::SimpleError;
//! use causality::{Actor, Cause, Effect};
//!
//! enum Command {
//!     TestDoor {actor_id: u32, actor_version: u8},
//! }
//!
//! impl Cause for Command {
//!     type ActorId = u32;
//!     type ActorVersion = u8;
//!     fn actor_id(&self) -> Self::ActorId {
//!         match self {
//!             Command::TestDoor {actor_id, ..} => {
//!                 *actor_id
//!             }
//!         }
//!     }
//!     fn actor_version(&self) -> Self::ActorVersion {
//!         match self {
//!             Command::TestDoor {actor_version, ..} => {
//!                 *actor_version
//!             }
//!         }
//!     }
//! }
//!
//! enum Event {
//!     Opened {version: u8, key: u32},
//!     Closed {version: u8, key: u32}
//! }
//!
//! impl Effect for Event {
//!     type Version = u8;
//!     type Key = u32;
//!     fn version(&self) -> Self::Version {
//!         match self {
//!             Event::Opened {version, ..} |
//!             Event::Closed {version, ..} => {
//!                 *version
//!             }
//!         }
//!     }
//!     fn key(&self) -> Self::Key {
//!         match self {
//!             Event::Opened {key, ..} |
//!             Event::Closed {key, ..} => {
//!                 *key
//!             }
//!         }
//!     }
//! }
//!
//! struct Door {
//!     id: u32,
//!     version: u8
//! }
//!
//! impl Actor<Command, Event, SimpleError> for Door {
//!     type Id = u32;
//!     type Version = u8;
//!     fn handle(&self, command: Command) -> Result<Vec<Event>, SimpleError> {
//!         match command {
//!             Command::TestDoor {actor_id, actor_version} => {
//!                 return Ok(vec![
//!                     Event::Opened {version: 1, key: 1},
//!                     Event::Closed {version: 1, key: 2}
//!                 ]);
//!             }
//!         }
//!         Err(SimpleError::new("command should be found due to enum type"))
//!     }
//!     fn apply(&mut self, effects: Vec<Event>) -> Result<(), SimpleError> {
//!         for effect in effects {
//!             match effect {
//!                 Event::Opened {key, ..} |
//!                 Event::Closed {key, ..} => {
//!                     self.version = key as u8;
//!                 }
//!             }
//!         }
//!         Ok(())
//!     }
//! }
//!
//! let mut door = Door {
//!     id: 1,
//!     version: 1
//! };
//! let command = Command::TestDoor {
//!     actor_id: 1,
//!     actor_version: 1
//! };
//! let events = door.handle(command).unwrap();
//! assert_eq!(events.len(), 2);
//! let result = door.apply(events);
//! assert!(result.is_ok());
//! ```

use std::error::Error;

/// Entity that handles `Causes` producing one or more `Effects` upon success.
///
/// Implemented for `Root Aggregates` or `Aggregates` in `Event Sourcing`.
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

/// Action performed on `Actor` that is expected to produce `Effects`.
///
/// Implemented for actions handled by `Actors`.
pub trait Cause {
    /// Unique `Actor` Id or aggregate key.
    type ActorId;
    /// Version of `Actor` handling `Cause` for ordering (optimistic concurrency or staleness).
    type ActorVersion;
    /// Returns unique `Actor` Id.
    fn actor_id(&self) -> Self::ActorId;
    /// Returns `Actor` version.
    fn actor_version(&self) -> Self::ActorVersion;
}

/// Event produced from `Actor` handling `Cause`.
///
/// Implemented for events produced by `Actors` handling `Causes`.
///
/// Expected that `Effects` are applied to `Actors` or `Aggregates` to represent state.
pub trait Effect {
    /// Schema version use to maintain backwards compatibility.
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
