use proc_macro2::TokenStream;

use crate::{schema, Ident};

/// We generate an InterfaceStruct for each interface in the schema.
///
/// These are output as empty structs that can be used as the TypeLock
/// in a SelectorStruct.
#[derive(Debug)]
pub struct InterfaceStruct {
    pub name: Ident,
}

impl InterfaceStruct {
    pub fn from_interface(interface: &schema::InterfaceType) -> Self {
        InterfaceStruct {
            name: Ident::for_type(&interface.name),
        }
    }
}

impl quote::ToTokens for InterfaceStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let name = &self.name;

        tokens.append_all(quote! {
            #[allow(dead_code)]
            pub struct #name {}
        });
    }
}
