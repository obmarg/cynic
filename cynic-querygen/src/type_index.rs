use graphql_parser::schema::{Definition, ScalarType};
use std::collections::HashMap;

use crate::{
    schema::{Document, Field, TypeDefinition},
    type_ext::TypeExt,
    Error,
};

pub struct TypeIndex<'a> {
    types: HashMap<&'a str, TypeDefinition<'a>>,
    root: String,
}

impl<'a> TypeIndex<'a> {
    pub fn from_schema(schema: &'a Document<'a>) -> TypeIndex<'a> {
        let types = schema
            .definitions
            .iter()
            .map(|definition| match definition {
                Definition::TypeDefinition(type_def) => {
                    Some((name_for_type(type_def), type_def.clone()))
                }
                _ => None,
            })
            .flatten()
            .collect::<HashMap<_, _>>();

        let mut rv = TypeIndex::default();
        rv.types.extend(types);

        for definition in &schema.definitions {
            if let Definition::SchemaDefinition(schema_def) = definition {
                if let Some(query) = schema_def.query {
                    rv.root = query.to_string();
                }
            }
        }

        rv
    }

    pub fn field_for_path<'b>(&'a self, path: &[&'b str]) -> Result<&'a Field<'a>, Error> {
        let root = self.types.get(self.root.as_str()).unwrap();
        if let TypeDefinition::Object(root_object) = root {
            self.find_field_recursive(&root_object.fields, self.root.as_str(), path)
        } else {
            Err(Error::ExpectedObject(self.root.clone()))
        }
    }

    /*
    pub fn type_for_path<'b>(&'a self, path: &[&'b str]) -> Result<&'a Type<'a, &'a str>, Error> {
        let root = self.types.get(self.root.as_str()).unwrap();
        if let TypeDefinition::Object(root_object) = root {
            Ok(&self
                .find_field_recursive(&root_object.fields, self.root.as_str(), path)?
                .field_type)
        } else {
            Err(Error::ExpectedObject(self.root.clone()))
        }
    }*/

    pub fn lookup_type(&self, name: &str) -> Option<&TypeDefinition<'a>> {
        self.types.get(name)
    }

    fn find_field_recursive<'b>(
        &'a self,
        fields: &'a [Field<'a>],
        current_type_name: &'a str,
        path: &[&'b str],
    ) -> Result<&'a Field<'a>, Error> {
        let get_field = |name| fields.iter().find(|field| field.name == name);

        match path {
            [] => panic!("This shouldn't happen"),
            [first] => get_field(*first).ok_or(Error::UnknownField(
                first.to_string(),
                current_type_name.to_string(),
            )),
            [first, rest @ ..] => {
                let inner_name = get_field(first)
                    .ok_or(Error::UnknownField(
                        first.to_string(),
                        current_type_name.to_string(),
                    ))?
                    .field_type
                    .inner_name();

                let inner_type = self
                    .types
                    .get(inner_name)
                    .ok_or(Error::UnknownType(inner_name.to_string()))?;

                if let TypeDefinition::Object(object) = inner_type {
                    self.find_field_recursive(&object.fields, &inner_name, rest)
                } else {
                    Err(Error::ExpectedObject(inner_name.to_string()))
                }
            }
        }
    }

    pub fn root_type_name(&self) -> &str {
        &self.root
    }
}

impl<'a> Default for TypeIndex<'a> {
    fn default() -> TypeIndex<'a> {
        let mut types = HashMap::new();

        types.insert("String", TypeDefinition::Scalar(ScalarType::new("String")));
        types.insert("Int", TypeDefinition::Scalar(ScalarType::new("Int")));
        types.insert(
            "Boolean",
            TypeDefinition::Scalar(ScalarType::new("Boolean")),
        );
        types.insert("ID", TypeDefinition::Scalar(ScalarType::new("ID")));

        TypeIndex {
            root: "Query".into(),
            types,
        }
    }
}

fn name_for_type<'a>(type_def: &TypeDefinition<'a>) -> &'a str {
    match type_def {
        TypeDefinition::Scalar(inner) => &inner.name,
        TypeDefinition::Object(inner) => &inner.name,
        TypeDefinition::Interface(inner) => &inner.name,
        TypeDefinition::Union(inner) => &inner.name,
        TypeDefinition::Enum(inner) => &inner.name,
        TypeDefinition::InputObject(inner) => &inner.name,
    }
}
