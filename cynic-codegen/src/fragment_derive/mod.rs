use std::collections::HashMap;

use proc_macro2::{Span, TokenStream};
use quote::format_ident;

pub fn fragment_derive(ast: &syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    // TODO:
    // 1. Get the schema name, type & path to the DSL module from derive attributes.
    // 2. Parse the schema, erroring on the schema name attr span.
    // 3. Check the type exists in the schema, erroring in the type attr span.
    // 4. Check the derive is for a struct, erroring appropriately if not.
    // 5. Get each of the fields in the struct - their names and types.
    //      We should probably support additional attrs here to allow for name mapping.
    // 6. Check that each of the names exists in the GQL type, erroring on appropriate
    //      span if not.
    // 7. Strip additional attrs from the struct fields.
    // 8. Output the struct
    // 9. Output an implementation of QueryFragment that calls the DSL for each field.
    //    - Will probably need to generate new-like constructor functions.

    let struct_attrs = parse_struct_attrs(&ast.attrs)?;
    println!("{:?}", struct_attrs);

    /*
    let schema_data: GraphQLSchema = graphql_parser::schema::parse_schema(&schema)
        .unwrap()
        .into();
        */

    Ok(quote::quote! {})
}

#[derive(Debug)]
struct CynicAttributes {
    schema_path: String,
    query_module: String,
    graphql_type: String,
}

fn parse_struct_attrs(attrs: &Vec<syn::Attribute>) -> Result<CynicAttributes, syn::Error> {
    use syn::{spanned::Spanned, Lit, Meta, MetaList, MetaNameValue, NestedMeta};

    let cynic_meta = attrs
        .iter()
        .find(|a| a.path.is_ident(&format_ident!("cynic")))
        .ok_or(syn::Error::new(
            Span::call_site(),
            "cynic attribute not provided",
        ))
        .and_then(|attr| attr.parse_meta())?;

    let mut attr_map: HashMap<DeriveAttribute, String> = HashMap::new();

    if let Meta::List(MetaList { nested, .. }) = &cynic_meta {
        for meta in nested {
            if let NestedMeta::Meta(Meta::NameValue(MetaNameValue { path, lit, .. })) = meta {
                if let Some(ident) = path.get_ident() {
                    let attr_name = ident
                        .to_string()
                        .parse()
                        .map_err(|e| syn::Error::new(ident.span(), &e))?;

                    if let Lit::Str(lit_str) = lit {
                        attr_map.insert(attr_name, lit_str.value());
                    } else {
                        // TODO: Re-factor this into something nicer...
                        // Could probably return an Error enum and move the strings
                        // elsewhere.
                        // Could potentially also do this with combinators or similar..
                        return Err(syn::Error::new(
                            lit.span(),
                            "values in the cynic attribute should be string literals",
                        ));
                    }
                } else {
                    return Err(syn::Error::new(
                        path.span(),
                        "keys in the cynic attribute should be a single identifier",
                    ));
                }
            } else {
                return Err(syn::Error::new(
                    meta.span(),
                    "The cynic attribute accepts a list of key=\"value\" pairs",
                ));
            }
        }
    } else {
        return Err(syn::Error::new(
            cynic_meta.span(),
            "The cynic attribute accepts a list of key=\"value\" pairs",
        ));
    }

    let schema_path = attr_map
        .remove(&DeriveAttribute::SchemaPath)
        .ok_or(syn::Error::new(
            cynic_meta.span(),
            "Missing required attribute: schema_path",
        ))?;

    let query_module = attr_map
        .remove(&DeriveAttribute::QueryModule)
        .ok_or(syn::Error::new(
            cynic_meta.span(),
            "Missing required attribute: query_module",
        ))?;

    let graphql_type = attr_map
        .remove(&DeriveAttribute::GraphqlType)
        .ok_or(syn::Error::new(
            cynic_meta.span(),
            "Missing required attribute: graphql_type",
        ))?;

    Ok(CynicAttributes {
        schema_path,
        query_module,
        graphql_type,
    })
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum DeriveAttribute {
    SchemaPath,
    QueryModule,
    GraphqlType,
}

impl std::str::FromStr for DeriveAttribute {
    type Err = String;

    fn from_str(s: &str) -> Result<DeriveAttribute, String> {
        if s == "schema_path" {
            Ok(DeriveAttribute::SchemaPath)
        } else if s == "query_module" {
            Ok(DeriveAttribute::QueryModule)
        } else if s == "graphql_type" {
            Ok(DeriveAttribute::GraphqlType)
        } else {
            Err(format!("Unknown cynic attribute: {}.  Expected one of schema_path, query_module, or graphql_type", s))
        }
    }
}

/*
struct GraphQLSchema {
    types: HashMap<String, GraphQLType>,
}

struct FragmentImpl {
    target_struct: Ident,
    fields: Vec<(String, TypePath)>,
}
*/
