use proc_macro2::TokenStream;

use crate::{schema, type_index::TypeIndex, Ident};

use super::SelectorStruct;

/// We generate an InterfaceStruct for each interface in the schema.
///
/// These are output as empty structs that can be used as the TypeLock
/// in a SelectorStruct.
#[derive(Debug)]
pub struct InterfaceStruct {
    pub name: Ident,
    pub selector_struct: SelectorStruct,
}

impl InterfaceStruct {
    pub fn from_interface(interface: &schema::InterfaceType, type_index: &TypeIndex) -> Self {
        InterfaceStruct {
            name: Ident::for_type(&interface.name),
            selector_struct: SelectorStruct::new(&interface.name, &interface.fields, type_index),
        }
    }
}

impl quote::ToTokens for InterfaceStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let selector = &self.selector_struct;

        tokens.append_all(quote! { #selector });
    }
}
