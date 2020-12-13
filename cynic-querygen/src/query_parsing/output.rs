use inflector::Inflector;
use std::rc::Rc;
use uuid::Uuid;

use super::{normalisation::Variable, value::TypedValue};
use crate::schema::{EnumDetails, InputField, OutputFieldType};
use crate::Error;

pub struct Output<'query, 'schema> {
    pub query_fragments: Vec<QueryFragment<'query, 'schema>>,
    pub input_objects: Vec<InputObject<'schema>>,
    pub enums: Vec<EnumDetails<'schema>>,
    pub scalars: Vec<Scalar<'schema>>,
    pub argument_structs: Vec<(String, Rc<ArgumentStruct<'query, 'schema>>)>,
}

pub struct Scalar<'schema>(pub &'schema str);

#[derive(Debug, PartialEq)]
pub struct QueryFragment<'query, 'schema> {
    pub fields: Vec<OutputField<'query, 'schema>>,
    pub target_type: String,
    pub argument_struct_name: Option<String>,

    pub name: String,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArgumentStruct<'query, 'schema> {
    pub id: Uuid,
    pub(super) name: String,
    pub fields: Vec<ArgumentStructField<'query, 'schema>>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ArgumentStructField<'query, 'schema> {
    Variable(Variable<'query, 'schema>),
    NestedStruct(Rc<ArgumentStruct<'query, 'schema>>),
}

impl<'query, 'schema> ArgumentStructField<'query, 'schema> {
    pub fn name(&self) -> String {
        match self {
            ArgumentStructField::Variable(var) => var.name.to_string(),
            ArgumentStructField::NestedStruct(arg_struct) => arg_struct.name.to_snake_case(),
        }
    }

    pub fn type_spec(&self) -> String {
        match self {
            ArgumentStructField::Variable(var) => var.value_type.type_spec().to_string(),
            ArgumentStructField::NestedStruct(arg_struct) => arg_struct.name.clone(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct OutputField<'query, 'schema> {
    pub name: &'schema str,
    pub field_type: RustOutputFieldType,

    pub arguments: Vec<FieldArgument<'query, 'schema>>,
}

/// An OutputFieldType that has been given a rust-land name.  Allows for
/// the fact that there may be several rust structs that refer to the same schema
/// type.
#[derive(Debug, PartialEq)]
pub enum RustOutputFieldType {
    NamedType(String),
    ListType(Box<RustOutputFieldType>),
    NonNullType(Box<RustOutputFieldType>),
}

impl RustOutputFieldType {
    pub fn from_schema_type(
        schema_type: &OutputFieldType<'_>,
        inner_name: String,
    ) -> RustOutputFieldType {
        match schema_type {
            OutputFieldType::NonNullType(inner) => RustOutputFieldType::NonNullType(Box::new(
                RustOutputFieldType::from_schema_type(inner, inner_name),
            )),
            OutputFieldType::ListType(inner) => RustOutputFieldType::ListType(Box::new(
                RustOutputFieldType::from_schema_type(inner, inner_name),
            )),
            OutputFieldType::NamedType(_) => RustOutputFieldType::NamedType(inner_name),
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
                    "ID" => return "cynic::Id".into(),
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
        self.value.to_literal()
    }
}

#[derive(Debug, PartialEq)]
pub struct InputObject<'schema> {
    pub name: String,
    pub fields: Vec<InputField<'schema>>,
}
