use proc_macro2::TokenStream;

use crate::{Error, Ident};

#[derive(Debug)]
pub struct Params {
    schema_filename: String,
}

impl Params {
    fn new(schema_filename: String) -> Self {
        Params { schema_filename }
    }
}

impl syn::parse::Parse for Params {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input
            .parse::<syn::LitStr>()
            .map(|lit_str| Params::new(lit_str.value()))
    }
}

pub fn scalars_as_strings(input: Params) -> Result<TokenStream, Error> {
    use quote::quote;

    let schema = std::fs::read_to_string(&input.schema_filename)?;
    let scalars = scalars_from_schema(graphql_parser::schema::parse_schema(&schema)?);

    Ok(quote! {
        #(scalars),
    })
}

fn scalars_from_schema(schema: graphql_parser::schema::Document) -> Vec<StringScalar> {
    use graphql_parser::schema::{Definition, TypeDefinition};

    let mut rv = vec![];
    for definition in schema.definitions {
        match definition {
            Definition::TypeDefinition(TypeDefinition::Scalar(scalar)) => rv.push(scalar.into()),
            _ => {}
        }
    }

    rv
}

struct StringScalar {
    name: Ident,
}

impl From<graphql_parser::schema::ScalarType> for StringScalar {
    fn from(scalar: graphql_parser::schema::ScalarType) -> Self {
        StringScalar {
            name: Ident::for_type(scalar.name),
        }
    }
}

impl quote::ToTokens for StringScalar {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let name = &self.name;

        tokens.append_all(quote! {
             type #name = String;
        })
    }
}
