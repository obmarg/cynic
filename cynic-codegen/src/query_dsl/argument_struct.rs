use graphql_parser::schema::{self, InputValue};
use proc_macro2::TokenStream;
use std::collections::HashSet;

use crate::{Ident, StructField, TypeIndex};

// TODO: Generate some of these somewhere...

#[derive(Debug)]
pub struct ArgumentStruct {
    name: Ident,
    arguments: Vec<StructField>,
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
        ArgumentStruct {
            name: ArgumentStruct::name_for_field(&field.name, required),
            arguments: arguments
                .iter()
                .map(|arg| {
                    StructField::from_input_value(
                        &arg,
                        Ident::for_module("super").into(),
                        type_index,
                    )
                })
                .collect(),
            required,
        }
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
            .map(|a| proc_macro2::Literal::string(&a.name.to_string()))
            .collect();

        let num_args = proc_macro2::Literal::usize_unsuffixed(argument_names.len());

        let into_iter_impl = if self.required {
            quote! {
                vec![
                    #(
                        ::cynic::Argument::new_serialize(#argument_strings, self.#argument_names)
                    ),*
                ].into_iter()
            }
        } else {
            quote! {
                let mut args = Vec::with_capacity(#num_args);

                #(
                    if self.#argument_names.is_some() {
                        args.push(::cynic::Argument::new_serialize(#argument_strings, self.#argument_names));
                    }
                )*

                args.into_iter()
            }
        };

        tokens.append_all(quote! {
            #attrs
            pub struct #name {
                #(
                    #arguments,
                )*
            }

            impl IntoIterator for #name {
                type Item = ::cynic::Argument;
                type IntoIter = ::std::vec::IntoIter<::cynic::Argument>;

                fn into_iter(self) -> Self::IntoIter {
                    #into_iter_impl
                }
            }
        })
    }
}
