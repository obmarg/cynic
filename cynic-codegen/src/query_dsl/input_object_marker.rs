use proc_macro2::TokenStream;

use crate::Ident;

/// We generate an InputObject for each input_type in the schema.
///
/// These are output as empty structs that can be used as the TypeLock
/// in an impl of the InputType trait.
#[derive(Debug)]
pub struct InputObjectMarker {
    pub name: Ident,
}

impl InputObjectMarker {
    pub fn from_input_object(en: &graphql_parser::schema::InputObjectType) -> Self {
        InputObjectMarker {
            name: Ident::for_type(&en.name),
        }
    }
}

impl quote::ToTokens for InputObjectMarker {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let name = &self.name;

        tokens.append_all(quote! {
            pub struct #name {}
        });
    }
}
