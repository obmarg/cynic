use super::field_type::FieldType;
use crate::{generic_param::GenericParameter, schema, Ident, TypeIndex};

#[derive(Debug, Clone)]
pub struct FieldArgument {
    pub(crate) name: Ident,
    pub(crate) argument_type: FieldType,
    pub(crate) gql_name: String,
    pub(crate) gql_type: String,
}

impl FieldArgument {
    pub fn from_input_value(value: &schema::InputValue, type_index: &TypeIndex) -> Self {
        use crate::schema::TypeExt;

        FieldArgument {
            name: Ident::for_field(&value.name),
            argument_type: FieldType::from_schema_type(&value.value_type, type_index),
            gql_type: value.value_type.to_graphql_string(),
            gql_name: value.name.clone(),
        }
    }

    pub fn is_required(&self) -> bool {
        !self.argument_type.is_nullable()
    }

    pub fn generic_parameter(&self) -> Option<GenericParameter> {
        // TOOD: Ok, so this one may warrant a different approach for now?
        // Unless I'm happy banning `.into()` for Strings etc.?
        self.argument_type
            .generic_parameter(Ident::for_type(format!("{}T", self.name.rust_name())))
    }
}
