use proc_macro2::TokenStream;

use super::argument_struct::ArgumentStruct;
use super::field_selector::FieldSelector;
use crate::graphql_extensions::FieldExt;

use crate::{FieldType, Ident, TypeIndex, TypePath};

/// We generate a SelectorStruct for each queryable object in the schema.
///
///
/// When output from our macros these contain FieldSelector functions that
/// create the selection sets which make up a graphql query.
#[derive(Debug)]
pub struct SelectorStruct {
    pub name: Ident,
    pub fields: Vec<FieldSelector>,
    pub is_root: bool,
}

impl SelectorStruct {
    pub fn from_object(
        obj: &graphql_parser::schema::ObjectType,
        type_index: &TypeIndex,
        is_root: bool,
    ) -> Self {
        let name = Ident::for_type(&obj.name);

        let mut processed_fields = Vec::with_capacity(obj.fields.len());
        for field in &obj.fields {
            let required_args_struct_name = if !field.required_arguments().is_empty() {
                Some(TypePath::new(vec![
                    Ident::for_module(&obj.name),
                    ArgumentStruct::name_for_field(&field.name, true),
                ]))
            } else {
                None
            };

            let optional_args_struct_name = if !field.optional_arguments().is_empty() {
                Some(TypePath::new(vec![
                    Ident::for_module(&obj.name),
                    ArgumentStruct::name_for_field(&field.name, false),
                ]))
            } else {
                None
            };

            processed_fields.push(FieldSelector::for_field(
                &field.name,
                FieldType::from_schema_type(&field.field_type, TypePath::empty(), type_index),
                name.clone(),
                required_args_struct_name,
                optional_args_struct_name,
            ));
        }

        SelectorStruct {
            name: name.clone(),
            is_root,
            fields: processed_fields,
        }
    }
}

impl quote::ToTokens for SelectorStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let name = &self.name;
        let fields = &self.fields;
        let query_root = if self.is_root {
            quote! { impl ::cynic::QueryRoot for #name {} }
        } else {
            quote! {}
        };

        tokens.append_all(quote! {
            pub struct #name;

            impl #name {
                #(
                    #fields
                )*
            }

            #query_root
        });
    }
}
