use inflector::Inflector;
use std::rc::Rc;
use uuid::Uuid;

use super::{normalisation::Variable, value::TypedValue};
use crate::schema::{EnumDetails, InputField, OutputFieldType};
use crate::Error;

// TODO: NOt jazzed about the name of this file as
// graphql has types so it almost sounds like this module is dealing with those...

pub struct Output<'query, 'schema> {
    pub query_fragments: Vec<QueryFragment<'query, 'schema>>,
    pub input_objects: Vec<InputObject<'schema>>,

    // TODO: I want enums sorted by name & unique
    pub enums: Vec<EnumDetails<'schema>>,

    // TODO: Scalars I also want sorted by name & unique.
    pub scalars: Vec<Scalar<'schema>>,

    // TODO: Argument structs can be sorted by name.
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

/*
impl<'query, 'schema> ArgumentStruct<'query, 'schema> {
    fn from_variables(
        variables: Vec<Variable<'query, 'schema>>,
        query_name: Option<String>,
    ) -> Option<ArgumentStruct<'query, 'schema>> {
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
pub struct OutputField<'query, 'schema> {
    pub name: &'schema str,
    pub field_type: OutputFieldType<'schema>,

    pub arguments: Vec<FieldArgument<'query, 'schema>>,
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
        self.value.to_literal(&self.value.value_type())
    }
}

#[derive(Debug, PartialEq)]
pub struct InputObject<'schema> {
    pub name: String,
    pub fields: Vec<InputField<'schema>>,
}
