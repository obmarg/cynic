use std::collections::HashMap;

use crate::schema::{Definition, Document, TypeDefinition};

pub struct TypeIndex<'a> {
    //name_to_kind: HashMap<String, Kind>,
    types: HashMap<&'a str, &'a TypeDefinition>,
}

impl<'a> TypeIndex<'a> {
    pub fn empty() -> Self {
        TypeIndex {
            types: HashMap::new(),
        }
    }

    pub fn for_schema(document: &'a Document) -> Self {
        let mut types = HashMap::new();
        for definition in &document.definitions {
            match definition {
                Definition::TypeDefinition(type_def) => {
                    types.insert(name_for_type(type_def), type_def);
                }
                _ => {}
            }
        }

        TypeIndex { types }
    }

    pub fn lookup_type(&self, name: &str) -> Option<&'a TypeDefinition> {
        self.types.get(name).map(|d| *d)
    }

    pub fn is_scalar(&self, name: &str) -> bool {
        self.types
            .get(name)
            .map(|def| matches!(def, TypeDefinition::Scalar(_)))
            .unwrap_or(false)
    }

    pub fn is_enum(&self, name: &str) -> bool {
        self.types
            .get(name)
            .map(|def| matches!(def, TypeDefinition::Enum(_)))
            .unwrap_or(false)
    }

    pub fn is_input_object(&self, name: &str) -> bool {
        self.types
            .get(name)
            .map(|def| matches!(def, TypeDefinition::InputObject(_)))
            .unwrap_or(false)
    }
}

fn name_for_type(type_def: &TypeDefinition) -> &str {
    match type_def {
        TypeDefinition::Scalar(inner) => &inner.name,
        TypeDefinition::Object(inner) => &inner.name,
        TypeDefinition::Interface(inner) => &inner.name,
        TypeDefinition::Union(inner) => &inner.name,
        TypeDefinition::Enum(inner) => &inner.name,
        TypeDefinition::InputObject(inner) => &inner.name,
    }
}
