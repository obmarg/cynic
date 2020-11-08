use graphql_parser::query::{Definition, Document, OperationDefinition, Selection, SelectionSet};
use std::rc::Rc;

use super::{normalisation::OperationVariable, Value};
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
    pub argument_structs: Vec<ArgumentStruct<'schema, 'query>>,
}

pub struct Scalar<'schema>(pub &'schema str);

#[derive(Debug, PartialEq)]
pub struct QueryFragment<'schema, 'query> {
    pub fields: Vec<Field<'schema, 'query>>,
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

#[derive(Debug, PartialEq)]
pub struct ArgumentStruct<'schema, 'query> {
    pub name: String,
    pub fields: Vec<OperationVariable<'query, 'schema>>,
}

impl<'schema, 'query> ArgumentStruct<'schema, 'query> {
    fn from_variables(
        variables: Vec<OperationVariable<'query, 'schema>>,
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

#[derive(Debug, PartialEq)]
pub struct Field<'schema, 'query> {
    pub name: &'schema str,
    pub field_type: OutputFieldType<'schema>,

    pub arguments: Vec<FieldArgument<'schema, 'query>>,
}

#[derive(Debug, PartialEq)]
pub struct FieldArgument<'schema, 'query> {
    pub name: &'schema str,
    value: Value<'query>,
    field_type: InputFieldType<'schema>,
}

impl<'schema, 'query> FieldArgument<'schema, 'query> {
    pub fn new(
        name: &'schema str,
        value: Value<'query>,
        field_type: InputFieldType<'schema>,
    ) -> Self {
        FieldArgument {
            name,
            value,
            field_type,
        }
    }

    pub fn to_literal(&self, type_index: &TypeIndex) -> Result<String, Error> {
        self.value.to_literal(&self.field_type)
    }
}

#[derive(Debug, PartialEq)]
pub struct InputObject<'schema> {
    pub name: String,
    pub fields: Vec<InputField<'schema>>,
}
