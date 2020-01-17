use graphql_parser::schema;
use proc_macro2::TokenStream;
use std::collections::HashSet;

use crate::{Ident, StructField};

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
        required: bool,
        scalar_names: &HashSet<String>,
    ) -> ArgumentStruct {
        ArgumentStruct {
            name: ArgumentStruct::name_for_field(&field.name, required),
            arguments: field
                .arguments
                .iter()
                .map(|arg| {
                    StructField::from_input_value(
                        &arg,
                        Ident::for_module("super").into(),
                        scalar_names,
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

        tokens.append_all(quote! {
            #attrs
            pub struct #name {
                #(
                    #arguments,
                )*
            }
        })
    }
}
