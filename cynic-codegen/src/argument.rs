use graphql_parser::schema;
use proc_macro2::TokenStream;
use std::collections::HashSet;

use super::field_type::FieldType;
use super::ident::Ident;

#[derive(Debug)]
pub struct Argument {
    name: Ident,
    argument_type: FieldType,
}

impl Argument {
    pub fn from_input_value(value: &schema::InputValue, scalar_names: &HashSet<String>) -> Self {
        Argument {
            name: Ident::for_field(&value.name),
            argument_type: FieldType::from_schema_type(&value.value_type, scalar_names),
        }
    }

    pub fn is_required(&self) -> bool {
        !self.argument_type.is_nullable()
    }
}

impl quote::ToTokens for Argument {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let name = &self.name;
        let argument_type = &self.argument_type;

        tokens.append_all(quote! {
            #name: #argument_type
        })
    }
}
