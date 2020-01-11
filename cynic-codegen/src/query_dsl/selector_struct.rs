use proc_macro2::TokenStream;
use std::collections::HashSet;

use super::argument_struct::ArgumentStruct;
use super::field_selector::FieldSelector;
use super::field_type::FieldType;
use super::graphql_extensions::FieldExt;
use super::type_path::TypePath;
use crate::ident::Ident;

/// We generate a SelectorStruct for each queryable object in the schema.
///
///
/// When output from our macros these contain FieldSelector functions that
/// create the selection sets which make up a graphql query.
#[derive(Debug)]
pub struct SelectorStruct {
    name: Ident,
    fields: Vec<FieldSelector>,
}

impl SelectorStruct {
    pub fn from_object(
        obj: &graphql_parser::schema::ObjectType,
        scalar_names: &HashSet<String>,
    ) -> Self {
        let name = Ident::for_type(&obj.name);
        SelectorStruct {
            name: name.clone(),
            fields: obj
                .fields
                .iter()
                .map(|field| {
                    let required_args_struct_name = if !field.required_arguments().is_empty() {
                        Some(TypePath::new(vec![
                            Ident::for_module(&obj.name),
                            ArgumentStruct::name_for_field(field, true),
                        ]))
                    } else {
                        None
                    };

                    let optional_args_struct_name = if !field.optional_arguments().is_empty() {
                        Some(TypePath::new(vec![
                            Ident::for_module(&obj.name),
                            ArgumentStruct::name_for_field(field, false),
                        ]))
                    } else {
                        None
                    };

                    FieldSelector::for_field(
                        &field.name,
                        FieldType::from_schema_type(
                            &field.field_type,
                            TypePath::empty(),
                            scalar_names,
                        ),
                        name.clone(),
                        required_args_struct_name,
                        optional_args_struct_name,
                    )
                })
                .collect(),
        }
    }
}

impl quote::ToTokens for SelectorStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let name = &self.name;
        let fields = &self.fields;

        tokens.append_all(quote! {
            pub struct #name;

            impl #name {
                #(
                    #fields
                )*
            }
        });
    }
}
