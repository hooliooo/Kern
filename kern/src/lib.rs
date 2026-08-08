pub mod application;
pub mod building_blocks;
pub mod infrastructure;

pub use ddd_macros::*;

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
