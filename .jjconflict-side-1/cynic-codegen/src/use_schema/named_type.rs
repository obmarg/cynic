use std::borrow::Cow;

use proc_macro2::TokenStream;

use crate::schema::{markers::TypeMarkerIdent, types::Type};

pub struct NamedType<'a> {
    graphql_name: Cow<'a, str>,
    marker_ident: TypeMarkerIdent<'a>,
}

impl<'a> NamedType<'a> {
    pub fn from_def(def: &Type<'a>) -> Option<Self> {
        match def {
            // Note: Currently we only use the NamedType lookup for members
            // of interfaces & unions - so we specifically don't generate anything for
            // scalars, inputs or enums.
            Type::Scalar(_) => None,
            Type::InputObject(_) => None,
            Type::Enum(_) => None,

            Type::Object(def) => Some(NamedType {
                graphql_name: def.name.clone(),
                marker_ident: def.marker_ident(),
            }),
            Type::Interface(def) => Some(NamedType {
                graphql_name: def.name.clone(),
                marker_ident: def.marker_ident(),
            }),
            Type::Union(def) => Some(NamedType {
                graphql_name: def.name.clone(),
                marker_ident: def.marker_ident(),
            }),
        }
    }
}

impl quote::ToTokens for NamedType<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let target_struct = self.marker_ident.to_rust_ident();
        let graphql_name = proc_macro2::Literal::string(self.graphql_name.as_ref());

        tokens.append_all(quote! {
            impl cynic::schema::NamedType for #target_struct {
                const NAME: &'static str = #graphql_name;
            }
        });
    }
}
