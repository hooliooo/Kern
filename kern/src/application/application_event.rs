use crate::{
    Timestamp, TimestampExt,
    application::{
        environment::Environment,
        ids::{AuthorizedParty, CommandId},
    },
    building_blocks::domain_event::DomainEvent,
};

#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ApplicationEvent<T>
where
    T: DomainEvent,
{
    /// The unique identifier of the command that triggered the application. Used to track the event in logging
    command_id: String,
    /// The string identifier of the command
    command: String,
    /// The identifier of the client that issued the command
    authorized_party: String,
    /// The environment of the application
    environment: String,
    /// The timestamp of when the application event was emitted
    issued_at: Timestamp,
    /// The domain event
    domain_event: T,
}

impl<T> ApplicationEvent<T>
where
    T: DomainEvent,
{
    pub fn new(
        command_id: CommandId,
        command: String,
        authorized_party: AuthorizedParty,
        environment: Environment,
        domain_event: T,
    ) -> Self {
        Self {
            command_id: command_id.value().to_string(),
            command,
            authorized_party: authorized_party.value().to_string(),
            environment: environment.as_str().to_string(),
            issued_at: Timestamp::now(),
            domain_event,
        }
    }
}
