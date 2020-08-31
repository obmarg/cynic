use graphql_parser::query;
use inflector::Inflector;

mod query_parsing;
mod schema;
mod type_ext;
mod type_index;
mod value_ext;

use query_parsing::PotentialStruct;
use type_ext::TypeExt;
use type_index::{GraphPath, TypeIndex};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Query document not supported: {0}")]
    UnsupportedQueryDocument(String),

    #[error("could not parse query document: {0}")]
    QueryParseError(#[from] query::ParseError),

    #[error("could not parse schema document: {0}")]
    SchemaParseError(#[from] graphql_parser::schema::ParseError),

    #[error("could not find field `{0}` on `{1}`")]
    UnknownField(String, String),

    #[error("could not find enum `{0}`")]
    UnknownEnum(String),

    #[error("could not find type `{0}`")]
    UnknownType(String),

    #[error("expected type `{0}` to be an object")]
    ExpectedObject(String),

    #[error("found a literal object but the argument is not an InputObject")]
    ArgumentNotInputObject,

    #[error("couldn't find an argument named `{0}`")]
    UnknownArgument(String),

    #[error("an enum-like value was provided to an argument that is not an enum")]
    ArgumentNotEnum,
}

#[derive(Debug)]
pub struct QueryGenOptions {
    pub schema_path: String,
    pub query_module: String,
}

impl Default for QueryGenOptions {
    fn default() -> QueryGenOptions {
        QueryGenOptions {
            schema_path: "schema.graphql".into(),
            query_module: "query_dsl".into(),
        }
    }
}

pub fn document_to_fragment_structs(
    query: impl AsRef<str>,
    schema: impl AsRef<str>,
    options: &QueryGenOptions,
) -> Result<String, Error> {
    let schema = graphql_parser::parse_schema::<&str>(schema.as_ref())?;
    let query = graphql_parser::parse_query::<&str>(query.as_ref())?;

    let type_index = TypeIndex::from_schema(&schema);
    let possible_structs = query_parsing::parse_query_document(&query, &type_index)?;

    let mut lines = vec![];

    lines.push("#[cynic::query_module(".into());
    lines.push(format!("    schema_path = \"{}\",", options.schema_path));
    lines.push(format!("    query_module = \"{}\",", options.query_module));
    lines.push(")]\nmod queries {".into());
    lines.push(format!(
        "    use super::{{{}, types::*}};\n",
        options.query_module
    ));

    for st in possible_structs {
        match st {
            PotentialStruct::QueryFragment(fragment) => {
                let graphql_type = type_index.type_name_for_path(&fragment.path)?;

                let name = if fragment.path.is_root() {
                    // If the user has given the query a name we use that, otherwise use the root
                    // graphql_type name for this operation type
                    fragment.name.unwrap_or(graphql_type)
                } else {
                    // For non root types we use the GraphQL type name for the
                    // type _and_ the `graphql_type` param.
                    graphql_type
                };

                let argument_struct_param = if let Some(name) = fragment.argument_struct_name {
                    format!(", argument_struct = \"{}\"", name)
                } else {
                    "".to_string()
                };

                lines.push("    #[derive(cynic::QueryFragment, Debug)]".into());
                lines.push(format!(
                    "    #[cynic(graphql_type = \"{}\"{})]",
                    graphql_type, argument_struct_param
                ));
                lines.push(format!("    pub struct {} {{", name));

                for field in fragment.fields {
                    if !field.arguments.is_empty() {
                        let arguments_string = field
                            .arguments
                            .iter()
                            .map(|arg| {
                                Ok(format!(
                                    "{} = {}",
                                    arg.name.to_snake_case(),
                                    arg.to_literal(&type_index)?
                                ))
                            })
                            .collect::<Result<Vec<_>, Error>>()?
                            .join(", ");

                        lines.push(format!("        #[arguments({})]", arguments_string));
                    }
                    // TODO: print out arguments
                    lines.push(format!(
                        "        pub {}: {},",
                        field.name.to_snake_case(),
                        field.field_type.type_spec(&type_index)
                    ))
                }
                lines.push(format!("    }}\n"));
            }
            PotentialStruct::Enum(en) => {
                let type_name = en.def.name;
                lines.push("    #[derive(cynic::Enum, Clone, Copy, Debug)]".into());
                lines.push("    #[cynic(".into());
                lines.push(format!("        graphql_type = \"{}\",", type_name));
                lines.push("        rename_all = \"SCREAMING_SNAKE_CASE\"".into());
                lines.push("    )]".into());
                lines.push(format!("    pub enum {} {{", type_name.to_pascal_case()));

                for variant in &en.def.values {
                    lines.push(format!("        {},", variant.name.to_pascal_case()))
                }
                lines.push("    }\n".into());
            }
            PotentialStruct::ArgumentStruct(argument_struct) => {
                lines.push("    #[derive(cynic::FragmentArguments, Clone, Debug)]".into());
                lines.push(format!("    pub struct {} {{", argument_struct.name));

                for field in &argument_struct.fields {
                    lines.push(format!(
                        "        pub {}: {},",
                        field.name.to_snake_case(),
                        field.field_type.type_spec(&type_index)
                    ));
                }

                lines.push("    }\n".into());
            }
            PotentialStruct::InputObject(input_object) => {
                lines.push("    #[derive(cynic::InputObject, Clone, Debug)]".into());
                lines.push(format!(
                    "    #[cynic(graphql_type = \"{}\", rename_all=\"camelCase\")]",
                    input_object.name
                ));
                lines.push(format!("    pub struct {} {{", input_object.name));

                for field in input_object.fields {
                    lines.push(format!(
                        "        pub {}: {},",
                        field.name.to_snake_case(),
                        field.value_type.type_spec(&type_index)
                    ))
                }

                lines.push("    }\n".into());
            }
            PotentialStruct::Scalar(_) => {}
        }
    }
    lines.push("}\n".into());

    lines.push("#[cynic::query_module(".into());
    lines.push(format!("    schema_path = \"{}\",", options.schema_path));
    lines.push(format!("    query_module = \"{}\",", options.query_module));
    lines.push(")]\nmod types {".into());

    // Output any custom scalars we need.
    for def in &schema.definitions {
        match def {
            schema::Definition::TypeDefinition(schema::TypeDefinition::Scalar(scalar)) => {
                lines.push("    #[derive(cynic::Scalar, Debug)]".into());
                lines.push(format!(
                    "    pub struct {}(String);\n",
                    scalar.name.to_pascal_case()
                ));
            }
            _ => (),
        }
    }
    lines.push("}\n".into());

    lines.push(format!("mod {}{{", options.query_module));
    lines.push("    use super::types::*;".into());
    lines.push(format!(
        "    cynic::query_dsl!(\"{}\");",
        options.schema_path
    ));
    lines.push("}\n".into());

    Ok(lines.join("\n"))
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
