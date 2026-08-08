use crate::entity;
use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{Data, DeriveInput, Field, Ident, Type};

pub fn generate_aggregate(ast: DeriveInput) -> TokenStream {
    let entity_ast = ast.clone();
    let identity = ast.ident;
    let generics = ast.generics;

    let fields: Vec<Field> = match ast.data {
        Data::Struct(data) => data.fields.into_iter().collect(),
        _ => panic!("Not a struct"),
    };

    let id_field = fields
        .iter()
        .find(|field| field.ident.as_ref().unwrap() == "id")
        .expect("No 'id' field found.");

    let _ = fields
        .iter()
        .find(|field| field.ident.as_ref().unwrap() == "version")
        .expect("No 'version' field found.");

    let generate_id_attribute = id_field
        .attrs
        .clone()
        .into_iter()
        .find(|attribute| attribute.path().is_ident("generate_id"));

    let generated_id_quote = match generate_id_attribute {
        Some(attribute) => {
            let id_type = attribute.parse_args::<Type>().unwrap();
            let id_identity_name = identity.to_string() + "Id";
            let id_identity = Ident::new(id_identity_name.as_str(), Span::call_site());

            let serialize_quote = if cfg!(feature = "serde") {
                quote::quote!(
                    impl kern::serde::Serialize for #id_identity {
                        fn serialize<S>(
                            &self,
                            serializer: S,
                        ) -> ::core::result::Result<S::Ok, S::Error>
                        where
                            S: kern::serde::Serializer,
                        {
                            kern::serde::Serialize::serialize(&self.0, serializer)
                        }
                    }
                )
            } else {
                quote::quote!()
            };

            quote::quote!(
                // Generate the ID struct of the struct
                #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
                pub struct #id_identity(#id_type);

                impl #id_identity {
                    pub fn new(id: #id_type) -> Self {
                        Self(id)
                    }

                    pub fn value_as_ref(&self) -> &#id_type {
                        &self.0
                    }

                    pub fn value(self) -> #id_type {
                        self.0
                    }
                }

                impl From<#id_type> for #id_identity {
                    fn from(value: #id_type) -> Self {
                        Self(value)
                    }
                }

                impl kern::building_blocks::ids::AggregateId for #id_identity {}

                #serialize_quote
            )
        }
        None => quote::quote!(),
    };
    let entity_quote: proc_macro2::TokenStream = entity::generate_entity(entity_ast).into();
    let identity_name = super::to_snake_case(identity.clone().to_string());

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    quote::quote!(
        #generated_id_quote

        impl #impl_generics kern::building_blocks::aggregate::Aggregate for #identity #ty_generics #where_clause {

            fn version(&self) -> u32 {
                self.version
            }

            fn type_name() -> &'static str {
                #identity_name
            }

        }

        #entity_quote
    )
    .into()
}
