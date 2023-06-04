use crate::casings::CasingExt;

mod attr_output;
mod enums;
mod field;
mod indent;
mod inline_fragments;
mod input_object;
pub mod query_fragment;
mod variables_struct;

pub use {
    enums::Enum,
    indent::indented,
    inline_fragments::InlineFragments,
    input_object::{InputObject, InputObjectField},
    query_fragment::QueryFragment,
    variables_struct::{VariablesStruct, VariablesStructField, VariablesStructForDisplay},
};

use field::Field;

pub struct Output<'query, 'schema> {
    pub query_fragments: Vec<QueryFragment<'query, 'schema>>,
    pub inline_fragments: Vec<InlineFragments>,
    pub input_objects: Vec<InputObject<'schema>>,
    pub enums: Vec<Enum<'schema>>,
    pub scalars: Vec<Scalar<'schema>>,
    pub variables_structs: Vec<VariablesStruct<'query, 'schema>>,
}

pub struct Scalar<'schema> {
    pub name: &'schema str,
    pub schema_name: Option<String>,
}

impl std::fmt::Display for Scalar<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let graphql_name = self.name;
        let rust_name = self.name.to_pascal_case();

        writeln!(f, "#[derive(cynic::Scalar, Debug, Clone)]")?;

        if graphql_name != rust_name {
            writeln!(f, "#[cynic(graphql_type = \"{}\")]", graphql_name)?;
        }

        writeln!(f, "pub struct {}(pub String);", rust_name)
    }
}
