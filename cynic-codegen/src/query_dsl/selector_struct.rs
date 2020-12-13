use proc_macro2::TokenStream;

use super::field_selector::FieldSelector;
use super::selection_builder::FieldSelectionBuilder;

use crate::{
    schema::{self, FieldExt},
    FieldType, Ident, TypeIndex, TypePath,
};

/// We generate a SelectorStruct for each queryable object in the schema.
///
///
/// When output from our macros these contain FieldSelector functions that
/// create the selection sets which make up a graphql query.
#[derive(Debug)]
pub struct SelectorStruct {
    pub name: Ident,
    pub graphql_name: String,
    pub fields: Vec<FieldSelector>,
    pub selection_builders: Vec<FieldSelectionBuilder>,
}

impl SelectorStruct {
    pub fn from_object(obj: &schema::ObjectType, type_index: &TypeIndex) -> Self {
        let name = Ident::for_type(&obj.name);

        let mut processed_fields = Vec::with_capacity(obj.fields.len());
        let mut selection_builders = Vec::with_capacity(obj.fields.len());

        for field in &obj.fields {
            let field_type = FieldType::from_schema_type(&field.field_type, type_index);

            let selection_builder = FieldSelectionBuilder::for_field(
                &field.name,
                field_type.clone(),
                name.clone(),
                field.optional_arguments(),
                type_index,
            );

            processed_fields.push(FieldSelector::for_field(
                &field.name,
                field_type,
                name.clone(),
                Ident::for_module(&obj.name),
                field.required_arguments(),
                TypePath::new(vec![
                    Ident::for_module(&name.to_string()),
                    selection_builder.name.clone(),
                ]),
                type_index,
            ));

            selection_builders.push(selection_builder);
        }

        SelectorStruct {
            name,
            graphql_name: obj.name.clone(),
            fields: processed_fields,
            selection_builders,
        }
    }
}

impl quote::ToTokens for SelectorStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let name = &self.name;
        let fields = &self.fields;

        tokens.append_all(quote! {
            #[allow(dead_code)]
            pub struct #name;

            #[allow(dead_code)]
            impl #name {
                #(
                    #fields
                )*
            }
        });
    }
}
