use graphql_parser::schema;
use proc_macro2::TokenStream;
use std::collections::HashSet;

use super::field_type::FieldType;
use super::ident::Ident;
use super::type_path::TypePath;

#[derive(Debug)]
pub struct StructField {
    name: Ident,
    argument_type: FieldType,
}

impl StructField {
    pub fn from_input_value(
        value: &schema::InputValue,
        type_path: TypePath,
        scalar_names: &HashSet<String>,
    ) -> Self {
        StructField {
            name: Ident::for_field(&value.name),
            argument_type: FieldType::from_schema_type(&value.value_type, type_path, scalar_names),
        }
    }

    pub fn is_required(&self) -> bool {
        !self.argument_type.is_nullable()
    }
}

impl quote::ToTokens for StructField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let name = &self.name;
        let argument_type = &self.argument_type;

        tokens.append_all(quote! {
            #name: #argument_type
        })
    }
}
