use proc_macro2::TokenStream;

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

        let selector = if self.field_type.contains_scalar() {
            // We call the scalar selector for scalars
            quote! { ::cynic::selection_set::scalar() }
        } else {
            // Otherwise we pass in the fields that the function
            // we generate accept as an argument.
            quote! { fields }
        };
        let selector = self.field_type.selection_set_call(selector);

        if self.field_type.contains_scalar() {
            let field_type = self.field_type.to_tokens(None, TypePath::new_super());
            quote! {
                pub fn select(self) ->
                ::cynic::selection_set::SelectionSet<'static, #field_type, super::#type_lock> {
                    #[allow(unused_imports)]
                    use ::cynic::selection_set::{string, integer, float, boolean};

                    ::cynic::selection_set::field(#query_field_name, self.args, #selector)
                }
            }
        } else {
            let decodes_to = self.field_type.decodes_to(quote! { T });
            let argument_type_lock = self.field_type.as_type_lock(TypePath::new_super());

            quote! {
                pub fn select<'a, T: 'a + Send + Sync>(
                    self,
                    fields: ::cynic::selection_set::SelectionSet<'a, T, #argument_type_lock>
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

        let argument_generics = self.optional_args.iter().map(|optional_arg| {
            if let Some(param) = optional_arg.generic_parameter() {
                let param_tokens = param.to_tokens(TypePath::new_super());
                quote! { < #param_tokens >}
            } else {
                quote! {}
            }
        });

        let argument_types = self.optional_args.iter().map(|a| {
            let generic_inner_type = a.generic_parameter().map(|param| param.name);
            a.argument_type
                .to_tokens(generic_inner_type, TypePath::new_super())
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
                    pub fn #argument_names #argument_generics(
                        mut self, #argument_names: impl ::cynic::IntoArgument<#argument_types>
                    ) -> Self {
                        self.args.push(
                            ::cynic::Argument::new(
                                #argument_strings,
                                #argument_gql_types,
                                #argument_names.into_argument()
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
