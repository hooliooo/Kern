use proc_macro::TokenStream;
use syn::DeriveInput;

mod aggregate;
mod domain_event;
mod entity;
mod generate_fields;
mod request;
mod value_object;

/// Generates the required methods for the Aggregate struct
///
/// Generates the required equality and hashing logic that follows the `Entity` semantics
/// in the domain-driven design context
///
/// Add the `field` attributes to the properties you want to generate getters for
#[proc_macro_derive(Aggregate, attributes(generate_id, entity_id, field))]
pub fn aggregate_macro(item: TokenStream) -> TokenStream {
    // parse
    let ast: DeriveInput = syn::parse_macro_input!(item as DeriveInput);
    // generate
    aggregate::generate_aggregate(ast)
}

/// Generates the required equality and hashing logic that follows the `Entity` semantics
/// in the domain-driven design context
///
/// Add the `field` attributes to the properties you want to generate getters for
///
/// Make sure to import kern::Entity and kern:traits::entity::Entity
#[proc_macro_derive(Entity, attributes(entity_id, field))]
pub fn entity_macro(item: TokenStream) -> TokenStream {
    // parse
    let ast: DeriveInput = syn::parse_macro_input!(item as DeriveInput);
    // generate
    entity::generate_entity(ast)
}

/// Generates the required equality and hashing logic that follows the `Value Object` semantics
/// in the domain-driven design context
///
/// Add the `field` attributes to the properties you want to generate getters for
///
/// Make sure to import kern::ValueObject and kern:traits::value_object::ValueObject
#[proc_macro_derive(ValueObject, attributes(field))]
pub fn value_object_macro(item: TokenStream) -> TokenStream {
    // parse
    let ast: DeriveInput = syn::parse_macro_input!(item as DeriveInput);
    // generate
    value_object::generate_value_object(ast)
}

/// Generates the boilerplate code for a DomainEvent
#[proc_macro_derive(DomainEvent, attributes(field))]
pub fn domain_event_macro(item: TokenStream) -> TokenStream {
    // parse
    let ast: DeriveInput = syn::parse_macro_input!(item as DeriveInput);
    // generate
    domain_event::generate_domain_event(ast)
}

/// Generates the boilerplate code for a Request
#[proc_macro_derive(Request)]
pub fn request_macro(item: TokenStream) -> TokenStream {
    // parse
    let ast: DeriveInput = syn::parse_macro_input!(item as DeriveInput);
    // generate
    request::generate_request(ast)
}

/// Generates the boilerplate code for an AutheitcatedRequest
#[proc_macro_derive(AuthenticatedRequest)]
pub fn authenticated_request_macro(item: TokenStream) -> TokenStream {
    // parse
    let ast: DeriveInput = syn::parse_macro_input!(item as DeriveInput);
    // generate
    request::generate_authenticated_request(ast)
}

/// Turns a string into snake case
fn to_snake_case(name: String) -> String {
    let mut snake_case = String::new();
    let mut chars = name.chars().peekable();

    while let Some(c) = chars.next() {
        if c.is_uppercase() {
            if let Some(next_char) = chars.peek()
                && !snake_case.is_empty()
                && next_char.is_lowercase()
            {
                snake_case.push('_')
            }
            snake_case.push(c.to_ascii_lowercase())
        } else {
            snake_case.push(c)
        }
    }
    snake_case
}

/// `CreatedOrganization` becomes `created-organization`.
fn to_kebab_case(name: String) -> String {
    to_snake_case(name).replace('_', "-")
}

/// Drops a trailing `-event` or `-events`, so an `AccountEvent` enum contributes `account`
/// rather than `account-event` to the names of its variants.
fn strip_event_suffix(name: String) -> String {
    name.strip_suffix("-events")
        .or_else(|| name.strip_suffix("-event"))
        .map(str::to_owned)
        .unwrap_or(name)
}

const FIELD_ATTR: &str = "field";
const ENTITY_ID_ATTR: &str = "entity_id";
