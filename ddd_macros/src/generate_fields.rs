use proc_macro2::{Ident, TokenStream};
use syn::{Field, Generics};
use crate::FIELD_ATTR;

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
            let ty = &field.ty;
            quote::quote!(
                pub fn #field_ident(&self) -> &#ty {
                    &self.#field_ident
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