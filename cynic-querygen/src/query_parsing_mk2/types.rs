use crate::query::Value;
//use crate::schema::{self, EnumType, InputValue, ScalarTypeExt, Type, TypeDefinition};

#[derive(Debug, PartialEq)]
pub struct FieldArgument<'a> {
    pub name: &'a str,
    value: Value<'a>,
    /*
    input_value: &'a InputValue<'a>,
    argument_type: &'a TypeDefinition<'a>,
    */
}

impl<'a> FieldArgument<'a> {
    fn new(
        name: &'a str,
        value: Value<'a>,
        /*
        input_value: &'a InputValue<'a>,
        argument_type: &'a TypeDefinition<'a>,
        */
    ) -> Self {
        FieldArgument {
            name,
            value,
            /*
            input_value,
            argument_type,
            */
        }
    }

    /*
    pub fn to_literal(&self, type_index: &TypeIndex) -> Result<String, Error> {
        self.value
            .to_literal(self.input_value, self.argument_type, type_index)
    }
    */
}
