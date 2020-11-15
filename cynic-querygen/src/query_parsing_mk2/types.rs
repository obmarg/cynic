use graphql_parser::query::{Definition, Document, OperationDefinition, Selection, SelectionSet};
use inflector::Inflector;
use std::rc::Rc;
use uuid::Uuid;

use super::{normalisation::Variable, value::TypedValue};
use crate::schema::{
    self, EnumType, InputField, InputFieldType, InputValue, OutputFieldType, ScalarTypeExt, Type,
    TypeDefinition,
};
use crate::{value_ext::ValueExt, Error, GraphPath, TypeExt, TypeIndex};

pub struct Output<'schema, 'query> {
    pub query_fragments: Vec<QueryFragment<'schema, 'query>>,
    pub input_objects: Vec<InputObject<'schema>>,

    // TODO: I want enums sorted by name & unique
    pub enums: Vec<Enum<'schema>>,

    // TODO: Scalars I also want sorted by name & unique.
    pub scalars: Vec<Scalar<'schema>>,

    // TODO: Argument structs can be sorted by name.
    pub argument_structs: Vec<(String, Rc<ArgumentStruct<'schema, 'query>>)>,
}

pub struct Scalar<'schema>(pub &'schema str);

#[derive(Debug, PartialEq)]
pub struct QueryFragment<'schema, 'query> {
    pub fields: Vec<OutputField<'schema, 'query>>,
    pub target_type: String,
    pub argument_struct_name: Option<String>,

    pub name: String,
}

impl QueryFragment<'_, '_> {
    fn uses_arguments(&self) -> bool {
        self.fields.iter().any(|f| !f.arguments.is_empty())
    }
}

#[derive(Debug, PartialEq)]
pub struct Enum<'schema> {
    pub def: EnumType<'schema>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArgumentStruct<'schema, 'query> {
    pub id: Uuid,
    pub(super) name: String,
    pub fields: Vec<ArgumentStructField<'schema, 'query>>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ArgumentStructField<'schema, 'query> {
    Variable(Variable<'query, 'schema>),
    NestedStruct(Rc<ArgumentStruct<'schema, 'query>>),
}

impl<'schema, 'query> ArgumentStructField<'schema, 'query> {
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

/*
impl<'schema, 'query> ArgumentStruct<'schema, 'query> {
    fn from_variables(
        variables: Vec<Variable<'query, 'schema>>,
        query_name: Option<String>,
    ) -> Option<ArgumentStruct<'schema, 'query>> {
        if variables.is_empty() {
            return None;
        }

        Some(ArgumentStruct {
            name: format!("{}Arguments", query_name.unwrap_or("".to_string())),
            fields: variables,
        })
    }
}
*/

#[derive(Debug, PartialEq)]
pub struct OutputField<'schema, 'query> {
    pub name: &'schema str,
    pub field_type: OutputFieldType<'schema>,

    pub arguments: Vec<FieldArgument<'schema, 'query>>,
}

#[derive(Debug, PartialEq)]
pub struct FieldArgument<'schema, 'query> {
    pub name: &'schema str,
    value: TypedValue<'schema, 'query>,
}

impl<'schema, 'query> FieldArgument<'schema, 'query> {
    pub fn new(name: &'schema str, value: TypedValue<'schema, 'query>) -> Self {
        FieldArgument { name, value }
    }

    pub fn to_literal(&self, type_index: &TypeIndex) -> Result<String, Error> {
        // TODO: ditch type index
        self.value.value.to_literal(&self.value.value_type)
    }
}

#[derive(Debug, PartialEq)]
pub struct InputObject<'schema> {
    pub name: String,
    pub fields: Vec<InputField<'schema>>,
}
