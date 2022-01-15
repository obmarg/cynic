use proc_macro2::TokenStream;

use crate::{
    schema::{markers::MarkerIdent, types as schema},
    Ident,
};

/// Outputs `HasSubtype` implementations for the given types.
#[derive(Debug)]
pub struct SubtypeMarkers<'a> {
    pub subtype: MarkerIdent<'a>,
    pub supertypes: Vec<MarkerIdent<'a>>,
}

impl<'a> SubtypeMarkers<'a> {
    pub fn from_interface(iface: &schema::InterfaceType<'a>) -> Self {
        let marker = iface.marker_ident();

        Self {
            subtype: marker,
            supertypes: vec![marker],
        }
    }

    pub fn from_union(def: &schema::UnionType<'a>) -> Vec<Self> {
        let supertype = def.marker_ident();

        def.types
            .iter()
            .map(|ty| SubtypeMarkers {
                subtype: ty.marker_ident(),
                supertypes: vec![supertype],
            })
            .collect()
    }

    pub fn from_object(obj: &schema::ObjectType<'a>) -> Option<Self> {
        if obj.implements_interfaces.is_empty() {
            return None;
        }

        Some(Self {
            subtype: obj.marker_ident(),
            supertypes: obj
                .implements_interfaces
                .iter()
                .map(|iface| iface.marker_ident())
                .collect(),
        })
    }
}

impl quote::ToTokens for SubtypeMarkers<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let subtype = proc_macro2::Ident::from(self.subtype);
        let supertypes = self
            .supertypes
            .iter()
            .copied()
            .map(proc_macro2::Ident::from);

        tokens.append_all(quote! {
            #(
                impl ::cynic::schema::HasSubtype<#subtype> for #supertypes {}
            )*
        });
    }
}
