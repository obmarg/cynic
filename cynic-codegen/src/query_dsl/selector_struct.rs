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
    pub argument_structs: Vec<ArgumentStruct>,
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
        let mut argument_structs = Vec::with_capacity(2 * obj.fields.len());

        for field in &obj.fields {
            let required_args = if !field.required_arguments().is_empty() {
                let args = ArgumentStruct::from_field(
                    field,
                    &field.required_arguments(),
                    true,
                    &type_index,
                );
                argument_structs.push(args.clone());
                Some(args)
            } else {
                None
            };

            let optional_args = if !field.optional_arguments().is_empty() {
                let args = ArgumentStruct::from_field(
                    field,
                    &field.optional_arguments(),
                    false,
                    &type_index,
                );
                argument_structs.push(args.clone());
                Some(args)
            } else {
                None
            };

            processed_fields.push(FieldSelector::for_field(
                &field.name,
                FieldType::from_schema_type(&field.field_type, TypePath::empty(), type_index),
                name.clone(),
                Ident::for_module(&obj.name),
                required_args,
                optional_args,
            ));
        }

        SelectorStruct {
            name: name.clone(),
            is_root,
            fields: processed_fields,
            argument_structs,
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
