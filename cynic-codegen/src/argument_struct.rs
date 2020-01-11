use graphql_parser::schema;
use proc_macro2::TokenStream;
use std::collections::HashSet;

use super::argument::Argument;
use super::ident::Ident;

// TODO: Generate some of these somewhere...

#[derive(Debug)]
pub struct ArgumentStruct {
    name: Ident,
    arguments: Vec<Argument>,
}

impl ArgumentStruct {
    pub fn name_for_field(field: &schema::Field, required: bool) -> Ident {
        let postfix = if required { "Args" } else { "OptionalArgs" };
        Ident::for_type(&format!("{}{}", field.name, postfix))
    }

    pub fn from_field(
        field: &schema::Field,
        required: bool,
        scalar_names: &HashSet<String>,
    ) -> ArgumentStruct {
        ArgumentStruct {
            name: ArgumentStruct::name_for_field(&field, required),
            arguments: field
                .arguments
                .iter()
                .map(|arg| Argument::from_input_value(&arg, scalar_names))
                .collect(),
        }
    }
}

impl quote::ToTokens for ArgumentStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let name = &self.name;
        let arguments = &self.arguments;

        tokens.append_all(quote! {
            pub struct #name {
                #(
                    #arguments,
                )*
            }
        })
    }
}
