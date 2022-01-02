use proc_macro2::TokenStream;

use crate::{schema, Ident};

/// Outputs `HasSubtype` implementations for the given types.
#[derive(Debug)]
pub struct SubtypeMarkers {
    pub subtype: Ident,
    pub supertypes: Vec<Ident>,
}

impl SubtypeMarkers {
    pub fn from_interface(iface: &schema::InterfaceType) -> Self {
        let ident = Ident::for_type(&iface.name);

        Self {
            subtype: ident.clone(),
            supertypes: vec![ident],
        }
    }

    pub fn from_union(def: &schema::UnionType) -> Vec<Self> {
        let supertype = Ident::for_type(&def.name);

        def.types
            .iter()
            .map(|ty| SubtypeMarkers {
                subtype: Ident::for_type(ty),
                supertypes: vec![supertype.clone()],
            })
            .collect()
    }

    pub fn from_object(obj: &schema::ObjectType) -> Option<Self> {
        if obj.implements_interfaces.is_empty() {
            return None;
        }

        Some(Self {
            subtype: Ident::for_type(&obj.name),
            supertypes: obj
                .implements_interfaces
                .iter()
                .map(Ident::for_type)
                .collect(),
        })
    }
}

impl quote::ToTokens for SubtypeMarkers {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let subtype = &self.subtype;
        let supertypes = &self.supertypes;

        tokens.append_all(quote! {
            #(
                impl ::cynic::schema::HasSubtype<#subtype> for #supertypes {}
            )*
        });
    }
}
