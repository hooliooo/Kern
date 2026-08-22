pub mod application;
pub mod building_blocks;
pub mod infrastructure;

pub use ddd_macros::*;

#[cfg(all(feature = "chrono", feature = "time"))]
compile_error!("The features 'chrono' and 'time' are mutually exclusive. Please choose only one.");

#[cfg(not(any(feature = "chrono", feature = "time")))]
compile_error!("You must enable either 'chrono' or 'time' feature.");

/// Re-export of `serde`, so generated code and downstream crates can reach it without
/// declaring serde themselves.
#[cfg(feature = "serde")]
pub use serde;

#[cfg(feature = "validator")]
pub mod validator_extensions {
    use std::collections::HashSet;

    use crate::building_blocks::{
        aggregate::Aggregate,
        error::{domain_error::DomainError, error_detail::ErrorDetail},
    };
    use validator::ValidationErrors;

    pub trait ResultValidation<T: Aggregate> {
        fn to_domain_error(self) -> Result<T, DomainError>;
    }

    impl<T> ResultValidation<T> for Result<T, ValidationErrors>
    where
        T: Aggregate,
    {
        fn to_domain_error(self) -> Result<T, DomainError> {
            self.map_err(|err| {
                let errors = err
                    .0
                    .into_iter()
                    .filter_map(|(key, value)| {
                        if let validator::ValidationErrorsKind::Field(errors) = value {
                            Some((key, errors))
                        } else {
                            None
                        }
                    })
                    .flat_map(|(key, errors)| {
                        let type_name = T::type_name();
                        let error_key = {
                            let mut error_key =
                                String::with_capacity(16 + type_name.len() + key.len());
                            error_key.push_str("error.");
                            error_key.push_str(type_name);
                            error_key.push_str(".invalid-");
                            error_key.extend(
                                key.as_ref().chars().map(|c| if c == '_' { '-' } else { c }),
                            );
                            error_key
                        };
                        errors.into_iter().filter_map(move |error| {
                            error.message.as_deref().map(|message| {
                                ErrorDetail::new(
                                    error_key.clone(),
                                    format!("'{}' {}", key, message),
                                )
                            })
                        })
                    })
                    .collect::<HashSet<ErrorDetail>>();
                DomainError::multiple(errors)
            })
        }
    }
}

pub trait TimestampExt {
    fn now() -> Self;
    fn to_iso8601(&self) -> String;
}

#[cfg(feature = "chrono")]
pub type Timestamp = chrono::DateTime<chrono::Utc>;

#[cfg(feature = "chrono")]
impl TimestampExt for Timestamp {
    fn now() -> Self {
        chrono::Utc::now()
    }

    fn to_iso8601(&self) -> String {
        self.to_rfc3339()
    }
}

#[cfg(feature = "time")]
pub type Timestamp = time::OffsetDateTime;

#[cfg(feature = "time")]
impl TimestampExt for Timestamp {
    fn now() -> Self {
        time::OffsetDateTime::now_utc()
    }

    fn to_iso8601(&self) -> String {
        self.format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default()
    }
}
