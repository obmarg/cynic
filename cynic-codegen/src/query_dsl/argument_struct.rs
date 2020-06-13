use proc_macro2::TokenStream;

use crate::field_argument::GenericParameter;
use crate::{
    schema::{self, InputValue},
    FieldArgument, Ident, TypeIndex, TypePath,
};

#[derive(Debug, Clone)]
pub struct ArgumentStruct {
    name: Ident,
    arguments: Vec<FieldArgument>,
    required: bool,
}

impl ArgumentStruct {
    pub fn name_for_field(field_name: &str, required: bool) -> Ident {
        let postfix = if required { "Args" } else { "OptionalArgs" };
        Ident::for_type(&format!("{}{}", field_name, postfix))
    }

    pub fn from_field(
        field: &schema::Field,
        arguments: &Vec<InputValue>,
        required: bool,
        type_index: &TypeIndex,
    ) -> ArgumentStruct {
        let name = ArgumentStruct::name_for_field(&field.name, required);
        ArgumentStruct {
            name,
            arguments: arguments
                .iter()
                .map(|arg| FieldArgument::from_input_value(&arg, type_index))
                .collect(),
            required,
        }
    }

    // Returns a list of the generic parameters this argument struct has
    pub fn generic_parameters(&self) -> Vec<GenericParameter> {
        self.arguments
            .iter()
            .map(|field| field.generic_parameter())
            .flatten()
            .collect()
    }

    pub fn type_tokens(&self, path_to_type: &Ident) -> TokenStream {
        use quote::quote;

        let generic_params = self
            .generic_parameters()
            .into_iter()
            .map(|param| param.name);

        let name = &self.name;

        quote! { #path_to_type :: #name < #(#generic_params),* > }
    }
}

impl quote::ToTokens for ArgumentStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let name = &self.name;
        let arguments = &self.arguments;
        let attrs = if self.required {
            quote! {}
        } else {
            quote! { #[derive(Default)] }
        };

        let argument_names: Vec<_> = arguments.iter().map(|a| a.name.clone()).collect();
        let argument_strings: Vec<_> = arguments
            .iter()
            .map(|a| proc_macro2::Literal::string(&a.gql_name.to_string()))
            .collect();
        let argument_gql_types: Vec<_> = arguments.iter().map(|a| a.gql_type.clone()).collect();

        let num_args = proc_macro2::Literal::usize_unsuffixed(argument_names.len());

        let into_iter_impl = if self.required {
            quote! {
                vec![
                    #(
                        ::cynic::Argument::from_serializable(
                            #argument_strings,
                            #argument_gql_types,
                            self.#argument_names
                        )
                    ),*
                ].into_iter()
            }
        } else {
            quote! {
                let mut args = Vec::with_capacity(#num_args);

                #(
                    if self.#argument_names.is_some() {
                        args.push(::cynic::Argument::from_serializable(
                            #argument_strings,
                            #argument_gql_types,
                            self.#argument_names
                        ));
                    }
                )*

                args.into_iter()
            }
        };

        let generic_param_defs = {
            let generic_defs = self
                .generic_parameters()
                .into_iter()
                .map(|param| param.to_tokens(Ident::for_module("super").into()));

            quote! { < #(#generic_defs,)*> }
        };

        let generic_param_names = {
            let names = self
                .generic_parameters()
                .into_iter()
                .map(|param| param.name);

            quote! { < #(#names,)* > }
        };

        let arguments = self.arguments.iter().map(|field| {
            let name = &field.name;
            let generic_inner_type = field.generic_parameter().map(|param| param.name);
            let type_tokens = field
                .argument_type
                .to_tokens(generic_inner_type, TypePath::empty());
            quote! { pub #name: #type_tokens }
        });

        tokens.append_all(quote! {
            #attrs
            #[allow(dead_code)]
            pub struct #name #generic_param_defs {
                #(
                    #arguments,
                )*
            }

            impl #generic_param_defs IntoIterator for #name #generic_param_names {
                type Item = ::cynic::Argument;
                type IntoIter = ::std::vec::IntoIter<::cynic::Argument>;

                fn into_iter(self) -> Self::IntoIter {
                    #into_iter_impl
                }
            }
        })
    }
}
