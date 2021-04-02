use proc_macro2::TokenStream;

use super::ArgumentParameterType;
use crate::{schema::InputValue, FieldArgument, FieldType, Ident, TypeIndex, TypePath};

/// A builder struct that is generated for each field in the query, to
/// allow optional arguments to be provided to that field.
///
/// Each field gets a FieldSelector function on their SelectorStruct
/// which returns one of these FieldSelectionBuilders.
#[derive(Clone, Debug)]
pub struct FieldSelectionBuilder {
    pub name: Ident,
    pub field_type: FieldType,
    pub type_lock: Ident,
    pub query_field_name: String,
    pub optional_args: Vec<FieldArgument>,
}

impl FieldSelectionBuilder {
    pub fn for_field(
        field_name: &str,
        field_type: FieldType,
        type_lock: Ident,
        optional_args: Vec<InputValue>,
        type_index: &TypeIndex,
    ) -> FieldSelectionBuilder {
        FieldSelectionBuilder {
            name: Ident::for_type(format!("{}SelectionBuilder", field_name)),
            field_type,
            type_lock,
            optional_args: optional_args
                .iter()
                .map(|v| FieldArgument::from_input_value(v, type_index))
                .collect(),
            query_field_name: field_name.to_string(),
        }
    }

    fn select_function_tokens(&self) -> TokenStream {
        use quote::quote;

        let query_field_name = &self.query_field_name;
        let type_lock = &self.type_lock;

        let arg_name = if self.field_type.contains_leaf_value() {
            Ident::for_field("inner")
        } else {
            Ident::for_field("fields")
        };
        let selector = self.field_type.selection_set_call(quote! { #arg_name });
        let decodes_to = self.field_type.decodes_to(quote! { T });
        let argument_type_lock = self.field_type.as_type_lock(TypePath::new_super());

        quote! {
            pub fn select<'a, T: 'a + Send + Sync>(
                self,
                #arg_name: ::cynic::selection_set::SelectionSet<'a, T, #argument_type_lock>
            ) -> ::cynic::selection_set::SelectionSet<'a, #decodes_to, super::#type_lock>
                {
                    ::cynic::selection_set::field(
                        #query_field_name,
                        self.args,
                        #selector
                    )
                }
        }
    }
}

impl quote::ToTokens for FieldSelectionBuilder {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let name = &self.name;

        let argument_names = self.optional_args.iter().map(|a| a.name.clone());
        let argument_strings = self
            .optional_args
            .iter()
            .map(|a| proc_macro2::Literal::string(&a.gql_name.to_string()));

        let argument_gql_types = self.optional_args.iter().map(|a| a.gql_type.clone());

        let argument_types = self.optional_args.iter().map(|a| {
            ArgumentParameterType::from_type(a.argument_type.clone())
                .to_tokens(TypePath::new_super())
                .unwrap()
        });

        let select_func = self.select_function_tokens();

        tokens.append_all(quote! {
            pub struct #name {
                args: Vec<::cynic::Argument>
            }

            impl #name {
                pub(super) fn new(args: Vec<::cynic::Argument>) -> Self {
                    #name { args }
                }

                #(
                    pub fn #argument_names(
                        mut self, #argument_names: #argument_types
                    ) -> Self {
                        self.args.push(
                            ::cynic::Argument::new(
                                #argument_strings,
                                #argument_gql_types,
                                ::cynic::serde_json::to_value(&#argument_names)
                            )
                        );

                        self
                    }
                )*

                #select_func
            }
        });
    }
}
