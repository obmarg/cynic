use proc_macro2::TokenStream;

pub(crate) mod input;

#[cfg(test)]
mod tests;

pub use input::ScalarDeriveInput;

use crate::Ident;

pub fn scalar_derive(ast: &syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    use darling::FromDeriveInput;

    match ScalarDeriveInput::from_derive_input(ast) {
        Ok(input) => scalar_derive_impl(input).or_else(|e| Ok(e.to_compile_error())),
        Err(e) => Ok(e.write_errors()),
    }
}

pub fn scalar_derive_impl(input: ScalarDeriveInput) -> Result<TokenStream, syn::Error> {
    use quote::quote;

    let schema_module = input.schema_module();

    // We're assuming that Darling has already validated this as a newtype enum,
    // so we can get away with panicing here.
    let field = input
        .data
        .take_struct()
        .expect("Expected enum")
        .into_iter()
        .next()
        .expect("Expected enum with one variant");

    let ident = input.ident;
    let inner_type = field.ty;
    let scalar_marker_ident = if let Some(graphql_type) = &input.graphql_type {
        Ident::new_spanned((**graphql_type).clone(), graphql_type.span())
    } else {
        Ident::for_type(ident.to_string()).with_span(ident.span())
    };
    let graphql_type_name = proc_macro2::Literal::string(
        &input
            .graphql_type
            .as_ref()
            .map(|s| (**s).clone())
            .unwrap_or_else(|| ident.to_string()),
    );

    Ok(quote! {
        #[automatically_derived]
        impl ::cynic::serde::Serialize for #ident {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::cynic::serde::Serializer,
            {
                self.0.serialize(serializer)
            }
        }

        #[automatically_derived]
        impl<'de> ::cynic::serde::Deserialize<'de> for #ident {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::cynic::serde::Deserializer<'de>,
            {
                #inner_type::deserialize(deserializer).map(Self)
            }
        }

        #[automatically_derived]
        impl ::cynic::schema::IsScalar<#schema_module::#scalar_marker_ident> for #ident {
            type SchemaType = #schema_module::#scalar_marker_ident;
        }

        #[automatically_derived]
        impl #schema_module::variable::Variable for #ident {
            const TYPE: ::cynic::variables::VariableType = ::cynic::variables::VariableType::Named(#graphql_type_name);
        }
    })
}
