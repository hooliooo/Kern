use proc_macro::TokenStream;
use syn::{Data, DeriveInput, Field};

pub fn generate_request(ast: DeriveInput) -> TokenStream {
    let identity = ast.ident;
    let generics = ast.generics;
    let fields: Vec<Field> = match ast.data {
        Data::Struct(data) => data.fields.into_iter().collect(),
        _ => panic!("Not a struct"),
    };

    let _ = fields
        .iter()
        .find(|field| field.ident.as_ref().unwrap() == "request_id")
        .expect("No 'request_id' field found.");

    let _ = fields
        .iter()
        .find(|field| field.ident.as_ref().unwrap() == "environment")
        .expect("No 'environment' field found.");

    let authorized_party_field = fields
        .iter()
        .find(|field| field.ident.as_ref().unwrap() == "authorized_party")
        .expect("No 'authorized_party' field found.");

    let authorized_party_type = &authorized_party_field.ty;

    let _ = fields
        .iter()
        .find(|field| field.ident.as_ref().unwrap() == "issued_at")
        .expect("No 'issued_at' field found.");

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    quote::quote!(

        impl #impl_generics kern::application::request::Request #ty_generics for #identity #where_clause {
            type RequestId = kern::application::ids::RequestId;
            type AuthorizedParty = #authorized_party_type;

            fn request_id(&self) -> &Self::RequestId {
                &self.request_id
            }

            fn environment(&self) -> &kern::application::environment::Environment {
                &self.environment
            }

            fn authorized_party(&self) -> &Self::AuthorizedParty {
                &self.authorized_party
            }

            fn issued_at(&self) -> &kern::Timestamp {
                &self.issued_at
            }
        }
    )
    .into()
}

pub fn generate_authenticated_request(ast: DeriveInput) -> TokenStream {
    let ast_clone = ast.clone();
    let identity = ast.ident;
    let generics = ast.generics;
    let fields: Vec<Field> = match ast.data {
        Data::Struct(data) => data.fields.into_iter().collect(),
        _ => panic!("Not a struct"),
    };

    let user_id_field = fields
        .iter()
        .find(|field| field.ident.as_ref().unwrap() == "user_id")
        .expect("No 'user_id' field found.");

    let user_id_type = &user_id_field.ty;

    let _ = fields
        .iter()
        .find(|field| field.ident.as_ref().unwrap() == "roles")
        .expect("No 'roles' field found.");

    let request_token_stream: proc_macro2::TokenStream = generate_request(ast_clone).into();

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    quote::quote!(

        #request_token_stream

        impl #impl_generics kern::application::request::AuthenticatedRequest #ty_generics for #identity #where_clause {

            type UserId = #user_id_type;

            fn user_id(&self) -> &Self::UserId {
                &self.user_id
            }

            fn roles(&self) -> &std::collections::HashSet<kern::application::role::Role> {
                &self.roles
            }
        }
    )
    .into()
}
