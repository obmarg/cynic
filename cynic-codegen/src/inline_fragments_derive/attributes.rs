use crate::attributes::{extract_meta_attrs, Attribute};

#[derive(Debug)]
pub struct InlineFragmentsDeriveAttributes {
    pub schema_path: Attribute,
    pub query_module: Attribute,
    pub graphql_type: Attribute,
    pub argument_struct: Option<Attribute>,
}

pub fn parse(attrs: &Vec<syn::Attribute>) -> Result<InlineFragmentsDeriveAttributes, syn::Error> {
    let (mut attr_map, attr_span) = extract_meta_attrs::<DeriveAttribute>(attrs)?;

    let schema_path = attr_map
        .remove(&DeriveAttribute::SchemaPath)
        .ok_or(syn::Error::new(
            attr_span,
            "Missing required attribute: schema_path",
        ))?;

    let query_module = attr_map
        .remove(&DeriveAttribute::QueryModule)
        .ok_or(syn::Error::new(
            attr_span,
            "Missing required attribute: query_module",
        ))?;

    let graphql_type = attr_map
        .remove(&DeriveAttribute::GraphqlType)
        .ok_or(syn::Error::new(
            attr_span,
            "Missing required attribute: graphql_type",
        ))?;

    let argument_struct = attr_map.remove(&DeriveAttribute::ArgumentStruct);

    Ok(InlineFragmentsDeriveAttributes {
        schema_path,
        query_module,
        graphql_type,
        argument_struct,
    })
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum DeriveAttribute {
    SchemaPath,
    QueryModule,
    GraphqlType,
    ArgumentStruct,
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
        } else if s == "argument_struct" {
            Ok(DeriveAttribute::ArgumentStruct)
        } else {
            Err(format!("Unknown cynic attribute: {}.  Expected one of schema_path, query_module, graphql_type or argument_struct", s))
        }
    }
}
