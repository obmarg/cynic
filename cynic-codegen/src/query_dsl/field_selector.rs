use proc_macro2::TokenStream;

use crate::{schema::InputValue, FieldArgument, FieldType, Ident, TypeIndex, TypePath};

/// A selection function for a field in our generated DSL
///
/// Each object in the schema will have one of these for each of it's
/// fields.  Calling this function will return a SelectionBuilder that
/// can be used to supply other arguments, and eventually build a
/// selection set.
#[derive(Debug)]
pub struct FieldSelector {
    pub rust_field_name: Ident,
    pub query_field_name: String,
    pub field_type: FieldType,
    pub type_lock: Ident,
    pub argument_structs_path: Ident,
    pub required_args: Vec<FieldArgument>,
    pub selection_builder: TypePath,
}

impl FieldSelector {
    pub fn for_field(
        name: &str,
        field_type: FieldType,
        type_lock: Ident,
        argument_structs_path: Ident,
        required_args: Vec<InputValue>,
        selection_builder: TypePath,
        type_index: &TypeIndex,
    ) -> FieldSelector {
        FieldSelector {
            rust_field_name: Ident::for_field(name),
            query_field_name: name.to_string(),
            field_type,
            type_lock,
            argument_structs_path,
            required_args: required_args
                .iter()
                .map(|v| FieldArgument::from_input_value(v, type_index))
                .collect(),
            selection_builder,
        }
    }
}

impl quote::ToTokens for FieldSelector {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let rust_field_name = &self.rust_field_name;

        let mut generic_params = Vec::new();

        let mut argument_defs = Vec::with_capacity(self.required_args.len());
        for arg in &self.required_args {
            // TODO: Move this into FieldArgument perhaps?
            let arg_name = &arg.name;

            if let Some(generic_param) = arg.generic_parameter() {
                generic_params.push(generic_param.to_tokens(TypePath::empty()));

                let arg_type = arg
                    .argument_type
                    .to_tokens(Some(generic_param.name), TypePath::empty());

                argument_defs.push(quote! { #arg_name: #arg_type });
            } else {
                let arg_type = arg.argument_type.to_tokens(None, TypePath::empty());

                argument_defs.push(quote! { #arg_name: #arg_type });
            }
        }

        let argument_names: Vec<_> = self.required_args.iter().map(|a| a.name.clone()).collect();
        let argument_strings: Vec<_> = self
            .required_args
            .iter()
            .map(|a| proc_macro2::Literal::string(&a.gql_name.to_string()))
            .collect();
        let argument_gql_types: Vec<_> = self
            .required_args
            .iter()
            .map(|a| a.gql_type.clone())
            .collect();

        let selection_builder = &self.selection_builder;

        tokens.append_all(quote! {
            pub fn #rust_field_name<#(#generic_params, )*>(
                #(#argument_defs, )*
            ) -> #selection_builder {
                #selection_builder::new(vec![
                    #(
                        ::cynic::Argument::from_serializable(
                            #argument_strings,
                            #argument_gql_types,
                            #argument_names
                        )
                    )*
                ])
            }
        })
    }
}
