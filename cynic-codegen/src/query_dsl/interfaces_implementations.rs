use proc_macro2::TokenStream;

use crate::{schema, Ident};

/// We generate an InterfaceImplementation for each type that implements interface.
///
/// These are output as `HasSubtype` implementations.
#[derive(Debug)]
pub struct InterfacesImplementations {
    pub implementor: Ident,
    pub interfaces: Vec<Ident>,
}

impl InterfacesImplementations {
    pub fn from_object(obj: &schema::ObjectType) -> Option<Self> {
        if obj.implements_interfaces.is_empty() {
            return None;
        }

        Some(Self {
            implementor: Ident::for_type(&obj.name),
            interfaces: obj
                .implements_interfaces
                .iter()
                .map(|interface| Ident::for_type(interface))
                .collect(),
        })
    }
}

impl quote::ToTokens for InterfacesImplementations {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let implementor = &self.implementor;
        let interfaces = &self.interfaces;

        tokens.append_all(quote! {
            #(
                impl ::cynic::selection_set::HasSubtype<#implementor> for #interfaces {}
            )*
        });
    }
}
