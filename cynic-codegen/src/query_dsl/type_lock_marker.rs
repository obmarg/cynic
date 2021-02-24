use proc_macro2::TokenStream;

use crate::{schema, Ident};

/// Outputs an empty struct that can be used a TypeLock for a given
/// enum or scalar impl
#[derive(Debug)]
pub struct TypeLockMarker {
    pub name: Ident,
}

impl TypeLockMarker {
    pub fn from_enum(en: &schema::EnumType) -> Self {
        TypeLockMarker {
            name: Ident::for_type(&en.name),
        }
    }

    pub fn from_scalar(scalar: &schema::ScalarType) -> Self {
        TypeLockMarker {
            name: Ident::for_type(&scalar.name),
        }
    }
}

impl quote::ToTokens for TypeLockMarker {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let name = &self.name;

        tokens.append_all(quote! {
            #[allow(dead_code)]
            pub struct #name {}
        });
    }
}
