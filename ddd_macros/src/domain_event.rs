use proc_macro::TokenStream;
use syn::{Data, DeriveInput};

use crate::{FIELD_ATTR, generate_fields::generate_fields};

/// Generates a `Serialize` impl for the event, or nothing when this crate's `serde` feature
/// (enabled by `kern/serde`) is off. Resolves serde through the `kern::serde` re-export.
fn generate_serialize(ast: &DeriveInput) -> proc_macro2::TokenStream {
    if !cfg!(feature = "serde") {
        return quote::quote!();
    }

    let identity = &ast.ident;
    let identity_name = identity.to_string();

    // Every type parameter has to carry a `Serialize` bound of its own, since the fields
    // holding it are what gets serialized.
    let mut generics = ast.generics.clone();
    for param in &mut generics.params {
        if let syn::GenericParam::Type(type_param) = param {
            type_param
                .bounds
                .push(syn::parse_quote!(kern::serde::Serialize));
        }
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let named_fields = |fields: &syn::Fields| -> Vec<syn::Ident> {
        fields
            .iter()
            .map(|field| {
                field
                    .ident
                    .clone()
                    .expect("DomainEvent only supports named fields")
            })
            .collect()
    };

    let body = match &ast.data {
        Data::Struct(data) => {
            let fields = named_fields(&data.fields);
            let names = fields.iter().map(|field| field.to_string());
            let count = fields.len();
            quote::quote!(
                use kern::serde::ser::SerializeStruct;
                let mut state = serializer.serialize_struct(#identity_name, #count)?;
                #( state.serialize_field(#names, &self.#fields)?; )*
                state.end()
            )
        }
        Data::Enum(data) => {
            let arms = data.variants.iter().enumerate().map(|(index, variant)| {
                let variant_identity = &variant.ident;
                let variant_name = variant_identity.to_string();
                let fields = named_fields(&variant.fields);
                let names = fields.iter().map(|field| field.to_string());
                let count = fields.len();
                let index = index as u32;
                quote::quote!(
                    #identity::#variant_identity { #( #fields, )* } => {
                        use kern::serde::ser::SerializeStructVariant;
                        let mut state = serializer.serialize_struct_variant(
                            #identity_name,
                            #index,
                            #variant_name,
                            #count,
                        )?;
                        #( state.serialize_field(#names, #fields)?; )*
                        state.end()
                    }
                )
            });
            quote::quote!(
                match self {
                    #( #arms )*
                }
            )
        }
        _ => unreachable!(),
    };

    quote::quote!(
        impl #impl_generics kern::serde::Serialize for #identity #ty_generics #where_clause {
            fn serialize<__S>(&self, serializer: __S) -> ::core::result::Result<__S::Ok, __S::Error>
            where
                __S: kern::serde::Serializer,
            {
                #body
            }
        }
    )
}

pub fn generate_domain_event(ast: DeriveInput) -> TokenStream {
    let serialize_quote = generate_serialize(&ast);
    let identity = ast.ident;
    let generics = ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // 1. Extract the concrete type of 'aggregate_id' for the Associated Type
    let agg_id_type = match &ast.data {
        Data::Struct(s) => s
            .fields
            .iter()
            .find(|f| f.ident.as_ref().unwrap() == "aggregate_id")
            .map(|f| &f.ty)
            .expect("Struct must have 'aggregate_id' field"),
        Data::Enum(e) => {
            let first_var = e.variants.first().expect("Enum must have variants");
            first_var
                .fields
                .iter()
                .find(|f| f.ident.as_ref().unwrap() == "aggregate_id")
                .map(|f| &f.ty)
                .expect("Enum variants must have 'aggregate_id' field")
        }
        _ => panic!("Only structs and enums are supported"),
    };

    // 2. Accessors for fields marked `#[field]`, as on `Aggregate` and `Entity`
    let fields_quote = match &ast.data {
        Data::Struct(data) => generate_fields(
            &identity,
            &generics,
            data.fields.iter().cloned().collect::<Vec<syn::Field>>(),
        ),
        Data::Enum(data) => {
            let marked = data.variants.iter().any(|variant| {
                variant
                    .fields
                    .iter()
                    .any(|field| field.attrs.iter().any(|a| a.path().is_ident(FIELD_ATTR)))
            });
            if marked {
                panic!("#[field] is not supported on enum variants");
            }
            quote::quote!()
        }
        _ => unreachable!(),
    };

    // 3. Generate the logic for each method. `event_type` is derived from the type name, or for
    // an enum the type and variant names, with a trailing `Event`/`Events` dropped.
    let (id_body, agg_id_body, agg_ver_body, occurred_body, event_type_body) = match &ast.data {
        Data::Struct(_) => {
            let event_type = super::strip_event_suffix(super::to_kebab_case(identity.to_string()));
            (
                quote::quote!(&self.id),
                quote::quote!(&self.aggregate_id),
                quote::quote!(self.aggregate_version),
                quote::quote!(&self.occurred_on),
                quote::quote!(#event_type),
            )
        }
        Data::Enum(data_enum) => {
            let variants: Vec<&syn::Ident> = data_enum.variants.iter().map(|v| &v.ident).collect();
            let prefix = super::strip_event_suffix(super::to_kebab_case(identity.to_string()));
            let event_types: Vec<String> = variants
                .iter()
                .map(|variant| {
                    let variant = super::to_kebab_case(variant.to_string());
                    if prefix.is_empty() {
                        variant
                    } else {
                        format!("{}-{}", prefix, variant)
                    }
                })
                .collect();
            (
                quote::quote!(match self { #( #identity::#variants { id, .. } => id, )* }),
                quote::quote!(match self { #( #identity::#variants { aggregate_id, .. } => aggregate_id, )* }),
                quote::quote!(match self { #( #identity::#variants { aggregate_version, .. } => *aggregate_version, )* }),
                quote::quote!(match self { #( #identity::#variants { occurred_on, .. } => occurred_on, )* }),
                quote::quote!(match self { #( #identity::#variants { .. } => #event_types, )* }),
            )
        }
        _ => unreachable!(),
    };

    quote::quote!(
        impl #impl_generics kern::building_blocks::domain_event::DomainEvent for #identity #ty_generics #where_clause where Self: Send + Sync + 'static {
            type Id = #agg_id_type;

            fn id(&self) -> &kern::building_blocks::ids::EventId {
                #id_body
            }

            fn aggregate_id(&self) -> &Self::Id {
                #agg_id_body
            }

            fn aggregate_version(&self) -> u32 {
                #agg_ver_body
            }

            fn occurred_on(&self) -> &kern::Timestamp {
                #occurred_body
            }

            fn event_type(&self) -> &'static str {
                #event_type_body
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }

        #fields_quote

        #serialize_quote
    ).into()
}
