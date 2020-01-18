use std::collections::HashMap;

use graphql_parser::schema::{Definition, Document, InputValue, TypeDefinition};

/// The kind of a GraphQL type
#[derive(Debug, PartialEq)]
pub enum Kind {
    Scalar,
    Enum,
    Object,
}

pub struct TypeIndex {
    name_to_kind: HashMap<String, Kind>,
}

impl TypeIndex {
    pub fn empty() -> Self {
        TypeIndex {
            name_to_kind: HashMap::new(),
        }
    }

    pub fn for_schema(document: &Document) -> Self {
        let mut name_to_kind = HashMap::new();
        for definition in &document.definitions {
            match definition {
                Definition::TypeDefinition(TypeDefinition::Scalar(scalar)) => {
                    name_to_kind.insert(scalar.name.clone(), Kind::Scalar);
                }
                Definition::TypeDefinition(TypeDefinition::Enum(en)) => {
                    name_to_kind.insert(en.name.clone(), Kind::Enum);
                }
                Definition::TypeDefinition(TypeDefinition::Object(object)) => {
                    name_to_kind.insert(object.name.clone(), Kind::Object);
                }

                _ => {}
            }
        }

        TypeIndex { name_to_kind }
    }

    pub fn is_scalar(&self, name: &str) -> bool {
        self.name_to_kind
            .get(name)
            .map(|kind| *kind == Kind::Scalar)
            .unwrap_or(false)
    }

    pub fn is_enum(&self, name: &str) -> bool {
        self.name_to_kind
            .get(name)
            .map(|kind| *kind == Kind::Enum)
            .unwrap_or(false)
    }
}
