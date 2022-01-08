use proc_macro2::TokenStream;

use crate::{idents::Ident, schema::TypeDefinition};

pub struct NamedType {
    ident: super::Ident,
}

impl NamedType {
    pub fn from_def(def: &TypeDefinition) -> Option<Self> {
        match def {
            // Note: Currently we only use the NamedType lookup for members
            // of interfaces & unions - so we specifically don't generate anything for
            // scalars, inputs or enums.
            TypeDefinition::Scalar(_) => None,
            TypeDefinition::InputObject(_) => None,
            TypeDefinition::Enum(_) => None,

            TypeDefinition::Object(def) => Some(NamedType {
                ident: Ident::for_type(&def.name),
            }),
            TypeDefinition::Interface(def) => Some(NamedType {
                ident: Ident::for_type(&def.name),
            }),
            TypeDefinition::Union(def) => Some(NamedType {
                ident: Ident::for_type(&def.name),
            }),
        }
    }
}

impl quote::ToTokens for NamedType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let rust_name = &self.ident;
        let graphql_name = proc_macro2::Literal::string(self.ident.graphql_name());

        tokens.append_all(quote! {
            impl ::cynic::schema::NamedType for #rust_name {
                fn name() -> &'static str {
                    #graphql_name
                }
            }
        });
    }
}
