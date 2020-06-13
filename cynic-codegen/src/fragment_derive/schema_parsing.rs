use std::collections::HashMap;

use crate::{schema, FieldType, Ident, TypeIndex, TypePath};

pub struct Schema {
    pub objects: HashMap<Ident, Object>,
}

impl From<schema::Document> for Schema {
    fn from(document: schema::Document) -> Self {
        use schema::{Definition, TypeDefinition};

        let type_index = TypeIndex::for_schema(&document);

        let mut objects = HashMap::new();

        for definition in document.definitions {
            match definition {
                Definition::TypeDefinition(TypeDefinition::Object(object)) => {
                    let object = Object::from_object(object, &type_index);
                    objects.insert(object.name.clone(), object);
                }
                _ => {}
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
    fn from_object(obj: schema::ObjectType, scalar_names: &TypeIndex) -> Object {
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

pub struct Field {
    pub arguments: HashMap<Ident, Argument>,
    pub name: Ident,
    pub field_type: FieldType,
}

impl Field {
    fn from_field(field: &schema::Field, type_index: &TypeIndex) -> Field {
        Field {
            name: Ident::for_field(&field.name),
            field_type: FieldType::from_schema_type(
                &field.field_type,
                TypePath::new(vec![]),
                type_index,
            ),
            arguments: field
                .arguments
                .iter()
                .map(|a| Argument::from_input_value(a, type_index))
                .map(|a| (a.name.clone(), a))
                .collect(),
        }
    }
}

pub struct Argument {
    pub name: Ident,
    pub required: bool,
}

impl Argument {
    fn from_input_value(value: &schema::InputValue, type_index: &TypeIndex) -> Argument {
        let argument_type =
            FieldType::from_schema_type(&value.value_type, Ident::new("").into(), type_index);
        Argument {
            name: Ident::for_field(&value.name),
            required: !argument_type.is_nullable(),
        }
    }
}
