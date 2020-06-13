use proc_macro2::TokenStream;

use crate::{schema, Ident};

/// We generate an EnumMarker for each enum in the schema.
///
/// These are output as empty structs that can be used as the TypeLock
/// in an impl of the Enum trait.
#[derive(Debug)]
pub struct EnumMarker {
    pub name: Ident,
}

impl EnumMarker {
    pub fn from_enum(en: &schema::EnumType) -> Self {
        EnumMarker {
            name: Ident::for_type(&en.name),
        }
    }
}

impl quote::ToTokens for EnumMarker {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let name = &self.name;

        tokens.append_all(quote! {
            #[allow(dead_code)]
            pub struct #name {}
        });
    }
}
