use std::fmt::Write;

use crate::casings::CasingExt;

use super::indented;
use crate::{query_parsing::TypedValue, schema::OutputFieldType, Error};

#[derive(Debug, PartialEq)]
pub struct QueryFragment<'query, 'schema> {
    pub fields: Vec<OutputField<'query, 'schema>>,
    pub target_type: String,
    pub variable_struct_name: Option<String>,

    pub name: String,
}

impl std::fmt::Display for QueryFragment<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "#[derive(cynic::QueryFragment, Debug)]")?;

        if self.target_type != self.name || self.variable_struct_name.is_some() {
            write!(f, "#[cynic(")?;
            if self.target_type != self.name {
                write!(f, "graphql_type = \"{}\"", self.target_type)?;
            }

            if let Some(name) = &self.variable_struct_name {
                if self.target_type != self.name {
                    write!(f, ", ")?;
                }
                write!(f, "variables = \"{}\"", name)?;
            }
            writeln!(f, ")]",)?;
        }

        writeln!(f, "pub struct {} {{", self.name)?;
        for field in &self.fields {
            write!(indented(f, 4), "{}", field)?;
        }

        writeln!(f, "}}")
    }
}

#[derive(Debug, PartialEq)]
pub struct OutputField<'query, 'schema> {
    pub name: &'schema str,
    pub rename: Option<&'schema str>,
    pub field_type: RustOutputFieldType,

    pub arguments: Vec<FieldArgument<'query, 'schema>>,
}

impl std::fmt::Display for OutputField<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.arguments.is_empty() {
            let arguments_string = self
                .arguments
                .iter()
                .map(|arg| Ok(format!("{}: {}", arg.name, arg.to_literal()?)))
                .collect::<Result<Vec<_>, Error>>()
                // TODO: This unwrap needs ditched somehow...
                .unwrap()
                .join(", ");

            writeln!(f, "#[arguments({})]", arguments_string)?;
        }

        let name = self.name.to_snake_case();
        let type_spec = self.field_type.type_spec();
        let mut output = super::Field::new(&name, &type_spec);

        if let Some(rename) = self.rename {
            output.add_rename(rename);
        }

        write!(f, "{}", output)
    }
}

/// An OutputFieldType that has been given a rust-land name.  Allows for
/// the fact that there may be several rust structs that refer to the same schema
/// type.
#[derive(Debug, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum RustOutputFieldType {
    NamedType(String),
    ListType(Box<RustOutputFieldType>),
    NonNullType(Box<RustOutputFieldType>),
}

impl RustOutputFieldType {
    pub fn from_schema_type(
        schema_type: &OutputFieldType<'_>,
        name_override: Option<String>,
    ) -> RustOutputFieldType {
        match schema_type {
            OutputFieldType::NonNullType(inner) => RustOutputFieldType::NonNullType(Box::new(
                RustOutputFieldType::from_schema_type(inner, name_override),
            )),
            OutputFieldType::ListType(inner) => RustOutputFieldType::ListType(Box::new(
                RustOutputFieldType::from_schema_type(inner, name_override),
            )),
            OutputFieldType::NamedType(type_ref) => RustOutputFieldType::NamedType(
                name_override
                    .or_else(|| {
                        let ty = type_ref.lookup().ok()?;
                        Some(ty.name().to_pascal_case())
                    })
                    .unwrap_or_else(|| "Unknown".to_string()),
            ),
        }
    }

    pub fn type_spec(&self) -> String {
        self.output_type_spec_imp(true)
    }

    fn output_type_spec_imp(&self, nullable: bool) -> String {
        if let RustOutputFieldType::NonNullType(inner) = self {
            return inner.output_type_spec_imp(false);
        }

        if nullable {
            return format!("Option<{}>", self.output_type_spec_imp(false));
        }

        match self {
            RustOutputFieldType::ListType(inner) => {
                format!("Vec<{}>", inner.output_type_spec_imp(true))
            }

            RustOutputFieldType::NonNullType(_) => panic!("NonNullType somehow got past an if let"),

            RustOutputFieldType::NamedType(s) => {
                match s.as_ref() {
                    "Int" => return "i32".into(),
                    "Float" => return "f64".into(),
                    "Boolean" => return "bool".into(),
                    // Technically the name is "ID" in graphql, but we've already pascal
                    // cased it
                    "Id" => return "cynic::Id".into(),
                    _ => {}
                }

                s.clone()
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct FieldArgument<'query, 'schema> {
    pub name: &'schema str,
    value: TypedValue<'query, 'schema>,
}

impl<'query, 'schema> FieldArgument<'query, 'schema> {
    pub fn new(name: &'schema str, value: TypedValue<'query, 'schema>) -> Self {
        FieldArgument { name, value }
    }

    pub fn to_literal(&self) -> Result<String, Error> {
        use crate::query_parsing::LiteralContext;

        self.value.to_literal(LiteralContext::Argument)
    }
}
