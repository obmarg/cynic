use graphql_parser::schema;
use proc_macro2::TokenStream;
use std::collections::HashSet;

use super::field_type::FieldType;
use super::type_path::TypePath;
use crate::{Ident, TypeIndex};

#[derive(Debug)]
pub struct StructField {
    pub(crate) name: Ident,
    pub(crate) argument_type: FieldType,
}

impl StructField {
    pub fn from_input_value(
        value: &schema::InputValue,
        type_path: TypePath,
        type_index: &TypeIndex,
    ) -> Self {
        StructField {
            name: Ident::for_field(&value.name),
            argument_type: FieldType::from_schema_type(&value.value_type, type_path, type_index),
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
            // TODO: Figure out if public is correct for _all_ the struct fields
            pub #name: #argument_type
        })
    }
}
