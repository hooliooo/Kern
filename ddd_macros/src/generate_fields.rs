use crate::FIELD_ATTR;
use proc_macro2::{Ident, TokenStream};
use syn::{Attribute, Field, GenericArgument, Generics, Meta, PathArguments, PathSegment, Type};

/// Whether the field opted into `#[field(copy)]`, returning a value rather than a reference.
fn wants_copy(attribute: &Attribute) -> bool {
    if let Meta::Path(_) = attribute.meta {
        return false;
    }

    let mut copy = false;
    if let Err(error) = attribute.parse_nested_meta(|meta| {
        if meta.path.is_ident("copy") {
            copy = true;
            Ok(())
        } else {
            Err(meta.error("unrecognised `#[field]` option, expected `copy`"))
        }
    }) {
        panic!("{}", error);
    }

    copy
}

fn is_str(ty: &Type) -> bool {
    matches!(ty, Type::Path(path) if path.qself.is_none() && path.path.is_ident("str"))
}

fn last_segment(ty: &Type) -> Option<&PathSegment> {
    match ty {
        Type::Path(path) if path.qself.is_none() => path.path.segments.last(),
        _ => None,
    }
}

/// The first type argument of a segment, skipping lifetimes, so `Cow<'a, str>` yields `str`.
fn first_type_argument(segment: &PathSegment) -> Option<&Type> {
    match &segment.arguments {
        PathArguments::AngleBracketed(arguments) => {
            arguments.args.iter().find_map(|argument| match argument {
                GenericArgument::Type(ty) => Some(ty),
                _ => None,
            })
        }
        _ => None,
    }
}

/// Whether a field of this type should be exposed as `&str`. `Rc<str>` and `Arc<str>` are
/// excluded so callers can still clone them for shared ownership.
fn is_str_like(ty: &Type) -> bool {
    let Some(segment) = last_segment(ty) else {
        return false;
    };

    match segment.ident.to_string().as_str() {
        "String" => segment.arguments.is_empty(),
        "Box" | "Cow" => first_type_argument(segment).is_some_and(is_str),
        _ => false,
    }
}

/// The item type of a `Vec<T>` or `Box<[T]>`, both exposed as `&[T]`.
fn slice_item(ty: &Type) -> Option<&Type> {
    let segment = last_segment(ty)?;

    match segment.ident.to_string().as_str() {
        "Vec" => first_type_argument(segment),
        "Box" => match first_type_argument(segment)? {
            Type::Slice(slice) => Some(&slice.elem),
            _ => None,
        },
        _ => None,
    }
}

fn option_item(ty: &Type) -> Option<&Type> {
    let segment = last_segment(ty)?;
    if segment.ident != "Option" {
        return None;
    }
    first_type_argument(segment)
}

/// The return type and body of a field's accessor. Owned containers are exposed as the
/// borrowed form they deref to; `Option` needs `as_deref` since `&Option<String>` does not
/// coerce to `Option<&str>`.
fn accessor(field_ident: &Ident, ty: &Type, copy: bool) -> (TokenStream, TokenStream) {
    if copy {
        return (quote::quote!(#ty), quote::quote!(self.#field_ident));
    }

    if is_str_like(ty) {
        return (quote::quote!(&str), quote::quote!(&self.#field_ident));
    }

    if let Some(item) = slice_item(ty) {
        return (quote::quote!(&[#item]), quote::quote!(&self.#field_ident));
    }

    if let Some(item) = option_item(ty) {
        if is_str_like(item) {
            return (
                quote::quote!(Option<&str>),
                quote::quote!(self.#field_ident.as_deref()),
            );
        }

        if let Some(item) = slice_item(item) {
            return (
                quote::quote!(Option<&[#item]>),
                quote::quote!(self.#field_ident.as_deref()),
            );
        }
    }

    (quote::quote!(&#ty), quote::quote!(&self.#field_ident))
}

pub fn generate_fields(identity: &Ident, generics: &Generics, fields: Vec<Field>) -> TokenStream {
    let getters = fields
        .into_iter()
        .filter(|field| {
            field
                .attrs
                .iter()
                .any(|attr| attr.path().is_ident(FIELD_ATTR))
        })
        .map(|field| {
            let field_ident = field.ident.as_ref().unwrap();
            let copy = field
                .attrs
                .iter()
                .filter(|attribute| attribute.path().is_ident(FIELD_ATTR))
                .any(wants_copy);
            let (return_type, body) = accessor(field_ident, &field.ty, copy);
            quote::quote!(
                pub fn #field_ident(&self) -> #return_type {
                    #body
                }
            )
        });

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    quote::quote!(
        impl #impl_generics #identity #ty_generics #where_clause {
            #(#getters)*
        }
    )
}
#[cfg(test)]
mod tests {
    use super::accessor;
    use proc_macro2::Ident;
    use syn::{Type, parse_quote};

    /// The generated return type and body, with whitespace removed for comparison.
    fn accessor_for(ty: Type) -> (String, String) {
        let field_ident: Ident = parse_quote!(value);
        let (return_type, body) = accessor(&field_ident, &ty, false);
        (
            return_type.to_string().replace(' ', ""),
            body.to_string().replace(' ', ""),
        )
    }

    #[test]
    fn owned_string_storage_is_exposed_as_str() {
        for ty in [
            parse_quote!(String),
            parse_quote!(std::string::String),
            parse_quote!(Box<str>),
            parse_quote!(Cow<'a, str>),
            parse_quote!(std::borrow::Cow<'static, str>),
        ] {
            assert_eq!(accessor_for(ty), ("&str".into(), "&self.value".into()));
        }
    }

    #[test]
    fn owned_sequences_are_exposed_as_slices() {
        assert_eq!(
            accessor_for(parse_quote!(Vec<String>)),
            ("&[String]".into(), "&self.value".into())
        );
        assert_eq!(
            accessor_for(parse_quote!(Box<[u8]>)),
            ("&[u8]".into(), "&self.value".into())
        );
    }

    /// `#[field(copy)]` wins over every borrowing rule, since the caller asked for a value.
    #[test]
    fn copy_fields_are_returned_by_value() {
        let field_ident: Ident = parse_quote!(value);

        for ty in [
            parse_quote!(bool),
            parse_quote!(u32),
            parse_quote!(OrganizationId),
            parse_quote!(Option<bool>),
        ] {
            let expected = quote::quote!(#ty).to_string().replace(' ', "");
            let (return_type, body) = accessor(&field_ident, &ty, true);
            assert_eq!(return_type.to_string().replace(' ', ""), expected);
            assert_eq!(body.to_string().replace(' ', ""), "self.value");
        }
    }

    /// `&Option<String>` does not coerce to `Option<&str>`, so these need `as_deref`.
    #[test]
    fn optional_storage_is_exposed_through_as_deref() {
        assert_eq!(
            accessor_for(parse_quote!(Option<String>)),
            ("Option<&str>".into(), "self.value.as_deref()".into())
        );
        assert_eq!(
            accessor_for(parse_quote!(Option<Box<str>>)),
            ("Option<&str>".into(), "self.value.as_deref()".into())
        );
        assert_eq!(
            accessor_for(parse_quote!(Option<Vec<u8>>)),
            ("Option<&[u8]>".into(), "self.value.as_deref()".into())
        );
    }

    #[test]
    fn everything_else_keeps_a_reference_to_its_own_type() {
        // Rc/Arc are excluded on purpose: callers may want to clone them for shared ownership.
        for (ty, expected) in [
            (parse_quote!(Rc<str>), "&Rc<str>"),
            (parse_quote!(Arc<str>), "&Arc<str>"),
            (parse_quote!(bool), "&bool"),
            (parse_quote!(Box<Path>), "&Box<Path>"),
            (parse_quote!(Option<u32>), "&Option<u32>"),
            (parse_quote!(HashMap<String, u32>), "&HashMap<String,u32>"),
        ] {
            let (return_type, body) = accessor_for(ty);
            assert_eq!(return_type, expected);
            assert_eq!(body, "&self.value");
        }
    }
}
