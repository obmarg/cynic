use std::collections::HashMap;

use crate::{schema, FieldType, Ident, TypeIndex};

pub struct Schema {
    pub objects: HashMap<Ident, Object>,
}

impl From<schema::Document> for Schema {
    fn from(document: schema::Document) -> Self {
        use schema::{Definition, TypeDefinition};

        let type_index = TypeIndex::for_schema(&document);

        let mut objects = HashMap::new();

        for definition in &document.definitions {
            if let Definition::TypeDefinition(TypeDefinition::Object(object)) = definition {
                let object = Object::from_object(object, &type_index);
                objects.insert(object.name.clone(), object);
            }
        }

        Schema { objects }
    }
}

pub struct Object {
    pub selector_struct: Ident,
    pub fields: HashMap<Ident, Field>,
    pub name: Ident,
}

impl Object {
    fn from_object(obj: &schema::ObjectType, scalar_names: &TypeIndex) -> Object {
        Object {
            selector_struct: Ident::for_type(&obj.name),
            fields: obj
                .fields
                .iter()
                .map(|f| Field::from_field(f, scalar_names))
                .map(|f| (f.name.clone(), f))
                .collect(),
            name: Ident::for_type(&obj.name),
        }
    }
}

#[derive(Debug)]
pub struct Field {
    pub arguments: Vec<Argument>,
    pub name: Ident,
    pub field_type: FieldType,
}

impl Field {
    fn from_field(field: &schema::Field, type_index: &TypeIndex) -> Field {
        Field {
            name: Ident::for_field(&field.name),
            field_type: FieldType::from_schema_type(&field.field_type, type_index),
            arguments: field
                .arguments
                .iter()
                .map(|a| Argument::from_input_value(a, type_index))
                .collect(),
        }
    }
}

#[derive(Debug)]
pub struct Argument {
    pub name: Ident,
    pub required: bool,
}

impl Argument {
    fn from_input_value(value: &schema::InputValue, type_index: &TypeIndex) -> Argument {
        let argument_type = FieldType::from_schema_type(&value.value_type, type_index);
        Argument {
            name: Ident::for_field(&value.name),
            required: !argument_type.is_nullable(),
        }
    }
}
