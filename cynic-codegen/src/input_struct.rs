use graphql_parser::schema;
use proc_macro2::TokenStream;
use std::collections::HashSet;

use crate::field_type::FieldType;
use crate::ident::Ident;
use crate::struct_field::StructField;
use crate::type_path::TypePath;

#[derive(Debug)]
pub struct InputStruct {
    name: Ident,
    fields: Vec<StructField>,
}

impl InputStruct {
    pub fn from_input_object(obj: schema::InputObjectType, scalar_names: &HashSet<String>) -> Self {
        InputStruct {
            name: Ident::for_type(&obj.name),
            fields: obj
                .fields
                .iter()
                .map(|field| StructField::from_input_value(&field, TypePath::empty(), scalar_names))
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
            pub struct #name {
                #(
                    #fields,
                )*
            }
        })
    }
}
