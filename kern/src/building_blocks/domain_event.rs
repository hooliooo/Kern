use crate::building_blocks::ids::{AggregateId, EventId};

/// ```
/// use kern::DomainEvent;
/// use kern::building_blocks::domain_event::DomainEvent;
/// use kern::building_blocks::ids::AggregateId;
/// use kern::building_blocks::ids::EventId;
/// use kern::building_blocks::ids::UserId;
/// use chrono::DateTime;
/// use chrono::Utc;
/// use uuid::Uuid;
/// use std::any::Any;
///
///
/// type AccountId = UserId<Uuid>;
///
/// #[derive(kern::DomainEvent, Debug)]
/// pub struct CreatedAccount {
///     id: EventId,
///     aggregate_id: AccountId,
///     aggregate_version: u32,
///     occurred_at: DateTime<Utc>
/// }
///
/// impl CreatedAccount {
///     pub fn new(aggregate_id: uuid::Uuid) -> Self {
///         Self {
///             id: EventId::new_random_v4(),
///             aggregate_id: AccountId::new(aggregate_id),
///             aggregate_version: 0,
///             occurred_at: Utc::now()
///         }
///     }
/// }
///
/// let a = CreatedAccount::new(uuid::Uuid::new_v4());
/// let b = CreatedAccount::new(uuid::Uuid::new_v4());
/// let boxed_a = Box::new(a);
/// let unboxed_a_as_any = boxed_a.as_any();
/// assert!(unboxed_a_as_any.is::<CreatedAccount>());
/// assert!(unboxed_a_as_any.downcast_ref::<CreatedAccount>().is_some());
///
/// let c = CreatedAccount::new(uuid::Uuid::new_v4());
/// let d = CreatedAccount::new(uuid::Uuid::new_v4());
///
/// #[derive(kern::DomainEvent, Debug)]
/// pub enum AccountEvent {
///     Created { id: EventId, aggregate_id: AccountId, aggregate_version: u32, occurred_at: DateTime<Utc> },
///     Updated { id: EventId, aggregate_id: AccountId, aggregate_version: u32, occurred_at: DateTime<Utc> },
/// }
///
/// let a = AccountEvent::Created { id: EventId::new_random_v4(), aggregate_id: AccountId::new(Uuid::new_v4()), aggregate_version: 0, occurred_at: Utc::now() };
/// let b = AccountEvent::Updated { id: EventId::new_random_v4(), aggregate_id: AccountId::new(Uuid::new_v4()), aggregate_version: 0, occurred_at: Utc::now() };
/// let boxed_a = Box::new(a);
/// let unboxed_a_as_any = boxed_a.as_any();
/// assert!(unboxed_a_as_any.is::<AccountEvent>());
/// if let Some(AccountEvent::Created { .. }) =
/// unboxed_a_as_any.downcast_ref::<AccountEvent>()
/// {
/// } else {
///     panic!("Expected Created event");
/// }
///
/// let boxed_b = Box::new(b);
/// let unboxed_b_as_any = boxed_b.as_any();
/// assert!(unboxed_b_as_any.is::<AccountEvent>());
/// if let Some(AccountEvent::Updated { .. }) =
/// unboxed_b_as_any.downcast_ref::<AccountEvent>()
/// {
/// } else {
///     panic!("Expected Updated event");
/// }
///
/// ```
pub trait DomainEvent {
    /// AggregateId type
    type Id: AggregateId;

    /// The unique identifier of the Domain Event
    fn id(&self) -> &EventId;

    /// The uniquie identifier of the Aggregate
    fn aggregate_id(&self) -> &Self::Id;

    /// The version of the Aggregate
    fn aggregate_version(&self) -> u32;

    /// The timestamp of when the domain event occurred
    fn occurred_at(&self) -> &chrono::DateTime<chrono::Utc>;

    fn as_any(&self) -> &dyn std::any::Any;
}

/// The DynDomainEvent trait is a type-erased version of DomainEvent so it can adhere to Rust'static
/// object safety rules
pub trait DynDomainEvent: Send + Sync {
    fn id(&self) -> &EventId;
    fn aggregate_id(&self) -> &dyn std::any::Any;
    fn aggregate_version(&self) -> u32;
    fn occurred_at(&self) -> &chrono::DateTime<chrono::Utc>;
    fn as_any(&self) -> &dyn std::any::Any;
}

impl<T> DynDomainEvent for T
where
    T: DomainEvent + Send + Sync + 'static,
{
    fn id(&self) -> &EventId {
        DomainEvent::id(self)
    }

    fn aggregate_id(&self) -> &dyn std::any::Any {
        DomainEvent::aggregate_id(self)
    }

    fn aggregate_version(&self) -> u32 {
        DomainEvent::aggregate_version(self)
    }

    fn occurred_at(&self) -> &chrono::DateTime<chrono::Utc> {
        DomainEvent::occurred_at(self)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
