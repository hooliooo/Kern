use crate::{
    Timestamp,
    building_blocks::ids::{AggregateId, EventId},
};

/// ```
/// use kern::DomainEvent;
/// use kern::building_blocks::domain_event::DomainEvent;
/// use kern::building_blocks::ids::AggregateId;
/// use kern::building_blocks::ids::EventId;
/// use kern::building_blocks::ids::UserId;
/// use kern::Timestamp;
/// use kern::TimestampExt;
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
///     // `#[field]` generates an accessor; the fields above already have one from the trait.
///     #[field]
///     holder: String,
///     // `copy` returns a value rather than a reference.
///     #[field(copy)]
///     active: bool,
///     occurred_on: Timestamp
/// }
///
/// impl CreatedAccount {
///     pub fn new(aggregate_id: uuid::Uuid) -> Self {
///         Self {
///             id: EventId::new_random_v4(),
///             aggregate_id: AccountId::new(aggregate_id),
///             aggregate_version: 0,
///             holder: "Ada".to_owned(),
///             active: true,
///             occurred_on: Timestamp::now()
///         }
///     }
/// }
///
/// let a = CreatedAccount::new(uuid::Uuid::new_v4());
/// let b = CreatedAccount::new(uuid::Uuid::new_v4());
///
/// // Derived from the type name.
/// assert_eq!(a.event_type(), "created-account");
/// assert_eq!(a.holder(), "Ada");
/// assert!(a.active());
///
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
///     Created { id: EventId, aggregate_id: AccountId, aggregate_version: u32, occurred_on: Timestamp },
///     Updated { id: EventId, aggregate_id: AccountId, aggregate_version: u32, occurred_on: Timestamp },
/// }
///
/// let a = AccountEvent::Created { id: EventId::new_random_v4(), aggregate_id: AccountId::new(Uuid::new_v4()), aggregate_version: 0, occurred_on: Timestamp::now() };
/// let b = AccountEvent::Updated { id: EventId::new_random_v4(), aggregate_id: AccountId::new(Uuid::new_v4()), aggregate_version: 0, occurred_on: Timestamp::now() };
///
/// // Named after the enum and the variant, with the trailing `Event` dropped.
/// assert_eq!(a.event_type(), "account-created");
/// assert_eq!(b.event_type(), "account-updated");
///
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
    fn occurred_on(&self) -> &Timestamp;

    /// The kebab-case name of the event, derived from the type's name. `CreatedAccount` gives
    /// `created-account`.
    fn event_type(&self) -> &'static str;

    fn as_any(&self) -> &dyn std::any::Any;
}

/// The DynDomainEvent trait is a type-erased version of DomainEvent so it can adhere to Rust'static
/// object safety rules
pub trait DynDomainEvent: Send + Sync {
    fn id(&self) -> &EventId;
    fn aggregate_id(&self) -> &dyn std::any::Any;
    fn aggregate_version(&self) -> u32;
    fn occurred_on(&self) -> &Timestamp;
    fn event_type(&self) -> &'static str;
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

    fn occurred_on(&self) -> &Timestamp {
        DomainEvent::occurred_on(self)
    }

    fn event_type(&self) -> &'static str {
        DomainEvent::event_type(self)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
