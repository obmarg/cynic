extern crate proc_macro;
use proc_macro2::TokenStream;
use std::collections::HashSet;

mod argument;
mod argument_struct;
mod field_selector;
mod field_type;
mod graphql_enum;
mod graphql_extensions;
mod ident;
mod selector_struct;

use argument_struct::ArgumentStruct;
use graphql_enum::GraphQLEnum;
use graphql_extensions::FieldExt;
use selector_struct::SelectorStruct;

#[derive(Debug, PartialEq)]
enum Error {
    IoError(String),
    ParseError(String),
}

impl From<graphql_parser::schema::ParseError> for Error {
    fn from(e: graphql_parser::schema::ParseError) -> Error {
        Error::ParseError(e.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::IoError(e.to_string())
    }
}

#[derive(Debug)]
struct QueryDslParams {
    schema_filename: String,
}

impl QueryDslParams {
    fn new(schema_filename: String) -> Self {
        QueryDslParams { schema_filename }
    }
}

impl syn::parse::Parse for QueryDslParams {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input
            .parse::<syn::LitStr>()
            .map(|lit_str| QueryDslParams::new(lit_str.value()))
    }
}

#[proc_macro]
pub fn query_dsl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as QueryDslParams);

    let rv = query_dsl_from_schema(input).unwrap().into();
    println!("{}", rv);
    rv
}

fn query_dsl_from_schema(input: QueryDslParams) -> Result<TokenStream, Error> {
    use quote::quote;

    let schema = std::fs::read_to_string(&input.schema_filename)?;
    let schema_data: GraphQLSchema = graphql_parser::schema::parse_schema(&schema)?.into();

    Ok(quote! {
        #schema_data
    })
}

#[derive(Debug)]
struct GraphQLSchema {
    selectors: Vec<SelectorStruct>,
    enums: Vec<GraphQLEnum>,
    argument_structs: Vec<ArgumentStruct>,
}

impl From<graphql_parser::schema::Document> for GraphQLSchema {
    fn from(document: graphql_parser::schema::Document) -> Self {
        use graphql_parser::schema::{Definition, TypeDefinition};

        let mut scalar_names = HashSet::new();

        for definition in &document.definitions {
            match definition {
                Definition::TypeDefinition(TypeDefinition::Scalar(scalar)) => {
                    scalar_names.insert(scalar.name.clone());
                }
                _ => {}
            }
        }

        let mut selectors = vec![];
        let mut enums = vec![];
        let mut argument_structs = vec![];

        for definition in document.definitions {
            match definition {
                Definition::TypeDefinition(TypeDefinition::Object(object)) => {
                    selectors.push(SelectorStruct::from_object(&object, &scalar_names));

                    for field in &object.fields {
                        let required_arguments = field.required_arguments();
                        if !required_arguments.is_empty() {
                            argument_structs.push(ArgumentStruct::from_field(
                                field,
                                true,
                                &scalar_names,
                            ));
                        }

                        let optional_arguments = field.optional_arguments();
                        if !optional_arguments.is_empty() {
                            argument_structs.push(ArgumentStruct::from_field(
                                field,
                                false,
                                &scalar_names,
                            ));
                        }
                    }
                }
                Definition::TypeDefinition(TypeDefinition::Enum(gql_enum)) => {
                    enums.push(gql_enum.into());
                }
                _ => {}
            }
        }

        GraphQLSchema {
            selectors,
            enums,
            argument_structs,
        }
    }
}

impl quote::ToTokens for GraphQLSchema {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let enums = &self.enums;
        let selectors = &self.selectors;
        let argument_structs = &self.argument_structs;

        tokens.append_all(quote! {
            #(
                #enums
            )*
            #(
                #selectors
            )*
            #(
                #argument_structs
            )*
        })
    }
}
