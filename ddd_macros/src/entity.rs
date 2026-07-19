use crate::generate_fields::generate_fields;
use crate::{ENTITY_ID_ATTR, FIELD_ATTR};
use proc_macro::TokenStream;
use syn::{Data, DeriveInput, Field};

pub fn generate_entity(ast: DeriveInput) -> TokenStream {
    let identity = ast.ident;
    let generics = ast.generics;
    let fields: Vec<Field> = match ast.data {
        Data::Struct(data) => data.fields.into_iter().collect(),
        _ => panic!("Not a struct"),
    };

    struct Fields(Option<Field>, Vec<Field>);
    let Fields(id_field, filtered_fields) =
        fields
            .into_iter()
            .fold(Fields(None, Vec::new()), |mut fields, field| {
                if field
                    .attrs
                    .iter()
                    .any(|attr| attr.path().is_ident(ENTITY_ID_ATTR))
                {
                    fields.0 = Some(field)
                } else if field
                    .attrs
                    .iter()
                    .any(|attr| attr.path().is_ident(FIELD_ATTR))
                {
                    fields.1.push(field)
                }
                fields
            });
    let id_field = id_field.expect("Missing `id` field");
    let id_field_type = id_field.ty;

    let getters = generate_fields(&identity, &generics, filtered_fields);

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    quote::quote!(

        impl #impl_generics kern::building_blocks::entity::Entity for #identity #ty_generics #where_clause {
            type Id = #id_field_type;

            fn id(&self) -> &Self::Id {
                &self.id
            }
        }

        impl #impl_generics PartialEq for #identity #ty_generics #where_clause {
            fn eq(&self, other: &Self) -> bool {
                self.id() == other.id()
            }
        }

        impl #impl_generics Eq for #identity #ty_generics #where_clause {}

        impl #impl_generics std::hash::Hash for #identity #ty_generics #where_clause {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.id.hash(state);
            }
        }

        #getters

    )
    .into()
}
