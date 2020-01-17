use std::collections::{HashMap, HashSet};

use crate::graphql_extensions::DocumentExt;
use crate::{FieldType, Ident, TypePath};

pub struct Schema {
    pub objects: HashMap<Ident, Object>,
}

impl From<graphql_parser::schema::Document> for Schema {
    fn from(document: graphql_parser::schema::Document) -> Self {
        use graphql_parser::schema::{Definition, TypeDefinition};

        let scalar_names = document.scalar_names();

        let mut objects = HashMap::new();

        for definition in document.definitions {
            match definition {
                Definition::TypeDefinition(TypeDefinition::Object(object)) => {
                    let object = Object::from_object(object, &scalar_names);
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
    fn from_object(
        obj: graphql_parser::schema::ObjectType,
        scalar_names: &HashSet<String>,
    ) -> Object {
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
    fn from_field(field: &graphql_parser::schema::Field, scalar_names: &HashSet<String>) -> Field {
        Field {
            name: Ident::for_field(&field.name),
            field_type: FieldType::from_schema_type(
                &field.field_type,
                TypePath::new(vec![]),
                scalar_names,
            ),
            arguments: field
                .arguments
                .iter()
                .map(Argument::from_input_value)
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
    fn from_input_value(value: &graphql_parser::schema::InputValue) -> Argument {
        // Note: passing a bad TypePath & scalar_names here intentionally,
        // so as to avoid having to pass them in.  Only need this FieldType
        // to check if argument is required...
        let argument_type =
            FieldType::from_schema_type(&value.value_type, Ident::new("").into(), &HashSet::new());
        Argument {
            name: Ident::for_field(&value.name),
            required: !argument_type.is_nullable(),
        }
    }
}
