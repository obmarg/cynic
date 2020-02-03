use graphql_parser::schema;
use proc_macro2::TokenStream;

use crate::{Ident, StructField, TypeIndex, TypePath};

#[derive(Debug)]
pub struct InputStruct {
    name: Ident,
    fields: Vec<StructField>,
}

impl InputStruct {
    pub fn from_input_object(obj: schema::InputObjectType, type_index: &TypeIndex) -> Self {
        InputStruct {
            name: Ident::for_type(&obj.name),
            fields: obj
                .fields
                .iter()
                .map(|field| StructField::from_input_value(&field, TypePath::empty(), type_index))
                .collect(),
        }
    }
}

impl quote::ToTokens for InputStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let name = &self.name;
        let fields = &self.fields;

        tokens.append_all(quote! {
            #[derive(Debug, ::serde::Serialize)]
            pub struct #name {
                #(
                    #fields,
                )*
            }
        })
    }
}
