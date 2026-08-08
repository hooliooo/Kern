use crate::building_blocks::value_object::ValueObject;
use std::hash::Hash;
use uuid::Uuid;

/// The AggregateId is the base trait that types should use when acting as the identity type for an Aggregate
pub trait AggregateId: Copy + Clone + Eq + PartialEq + Hash {}

/// The unique identifier of the Event
#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct EventId(Uuid);

impl EventId {
    /// Creates a new EventId
    /// # Arguments
    /// * `value` - The universally unique identifier (UUID)
    pub fn new(value: Uuid) -> Self {
        Self(value)
    }

    /// Creates a new, randomly generated EventId
    pub fn new_random_v4() -> Self {
        Self(uuid::Uuid::new_v4())
    }

    /// The unique identifier of the Event
    pub fn value(&self) -> &Uuid {
        &self.0
    }
}

impl ValueObject for EventId {}

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct UserId<T>(T);

impl<T> UserId<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }

    pub fn value(&self) -> &T {
        &self.0
    }
}

impl<T> From<T> for UserId<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> AggregateId for UserId<T> where T: Copy + Clone + Eq + PartialEq + Hash {}
impl<T> ValueObject for UserId<T> where T: Copy + Clone + Eq + PartialEq + Hash {}
