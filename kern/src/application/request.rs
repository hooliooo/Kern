use std::{collections::HashSet, hash::Hash};

use crate::{
    Timestamp,
    application::{environment::Environment, role::Role},
};

///
/// A Request is a Command that mutates an Aggregate or a Query that returns data. The trait defines the required metadata in a
/// Command-Query Responsibility Segregation (CQRS) architecture
/// ```
///
/// use kern::AuthenticatedRequest;
/// use kern::application::ids::RequestId;
/// use kern::application::request::Request;
/// use kern::application::request::AuthenticatedRequest;
/// use kern::application::role::Role;
/// use kern::building_blocks::ids::UserId;
/// use kern::Timestamp;
/// use kern::TimestampExt;
/// use std::collections::HashSet;
/// use uuid::Uuid;
///
/// #[derive(kern::AuthenticatedRequest, Debug)]
/// pub struct CreateAccount {
///     request_id: RequestId,
///     authorized_party: (),
///     environment: kern::application::environment::Environment,
///     issued_at: Timestamp,
///     user_id: UserId<Uuid>,
///     roles: HashSet<Role>
/// }
///
/// impl CreateAccount {
///     pub fn new(request_id: uuid::Uuid) -> Self {
///         Self {
///             request_id: RequestId::new(request_id),
///             authorized_party: (),
///             environment: kern::application::environment::Environment::Development,
///             issued_at: Timestamp::now(),
///             user_id: UserId::new(uuid::Uuid::new_v4()),
///             roles: HashSet::default()
///         }
///     }
/// }
///
/// let a = CreateAccount::new(uuid::Uuid::new_v4());
/// let b = CreateAccount::new(uuid::Uuid::new_v4());
///
/// let c = CreateAccount::new(uuid::Uuid::new_v4());
/// let d = CreateAccount::new(uuid::Uuid::new_v4());
/// ```
pub trait Request {
    type RequestId: Eq + PartialEq + Hash + Clone;
    /// The unique identifier of the Request
    fn request_id(&self) -> &Self::RequestId;

    type AuthorizedParty: Eq + PartialEq + Hash + Clone;
    /// The identifier of the client that issued the command
    fn authorized_party(&self) -> &Self::AuthorizedParty;

    /// The environment of the request. Useful for observability
    fn environment(&self) -> &Environment;

    /// The timestamp of when the request was issued
    fn issued_at(&self) -> &Timestamp;
}

pub trait AuthenticatedRequest: Request {
    type UserId: Eq + PartialEq + Hash + Clone;

    /// The user identifier
    fn user_id(&self) -> &Self::UserId;

    /// The roles of the user
    fn roles(&self) -> &HashSet<Role>;
}
