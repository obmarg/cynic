extern crate proc_macro;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::collections::{HashMap, HashSet};

#[derive(Debug, PartialEq)]
enum Error {
    IoError(String),
    ParseError(String),
    MissingQueryDefinition,
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
    use graphql_parser::schema::Definition;

    let schema = std::fs::read_to_string(&input.schema_filename)?;
    let schema_data = data_from_schema(graphql_parser::schema::parse_schema(&schema)?);

    let objects: Vec<_> = schema_data
        .types
        .iter()
        .map(|(_, v)| dsl_for_object(v, &schema_data.scalar_names))
        .collect();

    let enums: Vec<_> = schema_data.enums.iter().map(define_enum).collect();

    Ok(quote! {
        #(
            #enums
        )*
        #(
            #objects
        )*
    })
}

fn dsl_for_object(
    object: &graphql_parser::schema::ObjectType,
    scalar_names: &HashSet<String>,
) -> TokenStream {
    let struct_name = name_to_ident(&object.name);

    let function_tokens: Vec<_> = object
        .fields
        .iter()
        .map(|f| select_function_for_field(f, &struct_name, scalar_names))
        .collect();

    quote! {
        pub struct #struct_name;

        impl #struct_name {
            #(
                #function_tokens
            )*
        }
    }
}

fn define_enum(gql_enum: &graphql_parser::schema::EnumType) -> TokenStream {
    use inflector::Inflector;

    let enum_name = name_to_ident(&gql_enum.name);

    let enum_values = gql_enum
        .values
        .iter()
        .map(|v| name_to_ident(&v.name.to_pascal_case()));

    quote! {
        pub enum #enum_name {
            #(
                #enum_values
            ),*
        }
    }
}

fn name_to_ident(name: &str) -> proc_macro2::Ident {
    use quote::format_ident;
    if name == "type" {
        format_ident!("{}_", name)
    } else {
        format_ident!("{}", name)
    }
}

fn select_function_for_field(
    field: &graphql_parser::schema::Field,
    type_lock: &proc_macro2::Ident,
    scalar_names: &HashSet<String>,
) -> TokenStream {
    use graphql_parser::schema::Type;
    use inflector::Inflector;

    let query_field_name = syn::LitStr::new(&field.name, Span::call_site());
    let rust_field_name = name_to_ident(&field.name.to_snake_case());

    let (field_type, scalar_call) = field_type_and_scalar_call(&field.field_type, scalar_names);

    if let Some(scalar_call) = scalar_call {
        quote! {
            pub fn #rust_field_name() -> ::cynic::selection_set::SelectionSet<'static, #field_type, #type_lock> {
                use ::cynic::selection_set::{string, integer, float, boolean};

                ::cynic::selection_set::field(#query_field_name, #scalar_call)
            }
        }
    } else {
        quote! {
            pub fn #rust_field_name<'a, T>(fields: ::cynic::selection_set::SelectionSet<'a, T, #field_type>)
                -> ::cynic::selection_set::SelectionSet<T, #type_lock>
                where T: 'a {
                    ::cynic::selection_set::field(#query_field_name, fields)
                }
        }
    }
}

enum FieldType {
    Enum(syn::Type, String),
    Object(syn::Type),
    List(syn::Type),
}

fn field_type_and_scalar_call(
    gql_type: &graphql_parser::schema::Type,
    scalar_names: &HashSet<String>,
) -> (TokenStream, Option<TokenStream>) {
    use graphql_parser::schema::Type;

    match gql_type {
        Type::NonNullType(inner_type) => {
            let (inner_type, scalar_call) = field_type_and_scalar_call(inner_type, scalar_names);
            (
                quote! { Option<#inner_type> },
                scalar_call.map(|expr| quote! { ::cynic::selection_set::option(#expr) }),
            )
        }
        Type::ListType(inner_type) => {
            let (inner_type, scalar_call) = field_type_and_scalar_call(inner_type, scalar_names);
            (
                quote! { Vec<#inner_type> },
                scalar_call.map(|expr| quote! { ::cynic::selection_set::vec(#expr) }),
            )
        }
        Type::NamedType(name) => {
            let (field_type, scalar_func) = if name == "String" {
                ("String".to_string(), Some("string"))
            } else if name == "Int" {
                ("i64".to_string(), Some("integer"))
            } else if name == "Float" {
                ("f64".to_string(), Some("float"))
            } else if name == "Boolean" {
                ("bool".to_string(), Some("boolean"))
            } else if name == "ID" {
                // TODO: Could do something more sensible for IDs here...
                ("String".to_string(), Some("string"))
            } else if name == "JSON" {
                return (
                    quote! { ::serde_json::Value },
                    Some(quote! { ::cynic::selection_set::json() }),
                );
            } else if scalar_names.contains(name) {
                // We've got a custom scalar.
                let field_type = name_to_ident(&name);
                return (
                    quote! { #field_type },
                    Some(quote! { ::cynic::selection_set::scalar() }),
                );
            } else {
                (name.to_string(), None)
            };

            let field_type = name_to_ident(&field_type);
            let scalar_func = scalar_func.map(name_to_ident);

            (quote! { #field_type }, scalar_func.map(|f| quote! { #f() }))
        }
    }
}

#[derive(Debug)]
struct SchemaData {
    types: HashMap<String, graphql_parser::schema::ObjectType>,
    enums: Vec<graphql_parser::schema::EnumType>,
    scalar_names: HashSet<String>,
}

fn data_from_schema(document: graphql_parser::schema::Document) -> SchemaData {
    use graphql_parser::schema::{Definition, TypeDefinition};

    let mut types = HashMap::new();
    let mut enums = vec![];
    let mut scalar_names = HashSet::new();

    for definition in document.definitions {
        match definition {
            Definition::TypeDefinition(TypeDefinition::Object(object)) => {
                types.insert(object.name.clone(), object.clone());
            }
            Definition::TypeDefinition(TypeDefinition::Enum(gql_enum)) => {
                enums.push(gql_enum.clone());
            }
            Definition::TypeDefinition(TypeDefinition::Scalar(scalar)) => {
                scalar_names.insert(scalar.name.clone());
            }
            _ => {}
        }
    }

    SchemaData {
        types,
        enums,
        scalar_names,
    }
}
