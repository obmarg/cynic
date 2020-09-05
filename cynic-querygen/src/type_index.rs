use graphql_parser::schema::{Definition, ScalarType};
use std::collections::HashMap;

use crate::{
    schema::{Document, Field, TypeDefinition},
    type_ext::TypeExt,
    Error,
};

pub struct TypeIndex<'a> {
    types: HashMap<&'a str, TypeDefinition<'a>>,
    query_root: String,
    mutation_root: String,
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
                    rv.query_root = query.to_string();
                }
                if let Some(mutation) = schema_def.mutation {
                    rv.mutation_root = mutation.to_string();
                }
            }
        }

        rv
    }

    pub fn field_for_path<'b>(&'a self, path: &GraphPath) -> Result<&'a Field<'a>, Error> {
        let root_name = match path.operation_type {
            OperationType::Query => &self.query_root,
            OperationType::Mutation => &self.mutation_root,
        }
        .as_str();

        let root = self.types.get(root_name).unwrap();
        if let TypeDefinition::Object(root_object) = root {
            self.find_field_recursive(&root_object.fields, root_name, &path.path)
        } else {
            Err(Error::ExpectedObject(root_name.to_string()))
        }
    }

    // Looks up the name of the type at Path.
    pub fn type_name_for_path(&'a self, path: &GraphPath) -> Result<&'a str, Error> {
        match (path.is_root(), &path.operation_type) {
            (true, OperationType::Query) => Ok(&self.query_root),
            (true, OperationType::Mutation) => Ok(&self.mutation_root),
            (false, _) => Ok(self.field_for_path(path)?.field_type.inner_name()),
        }
    }

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
            query_root: "Query".into(),
            mutation_root: "Mutation".into(),
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

#[derive(Debug, PartialEq, Clone)]
enum OperationType {
    Query,
    Mutation,
}

/// The path to a type within a graphql graph.
#[derive(Debug, PartialEq, Clone)]
pub struct GraphPath<'a> {
    operation_type: OperationType,
    path: Vec<&'a str>,
}

impl<'a> GraphPath<'a> {
    pub fn for_mutation() -> Self {
        GraphPath {
            operation_type: OperationType::Mutation,
            path: Vec::new(),
        }
    }

    pub fn for_query() -> Self {
        GraphPath {
            operation_type: OperationType::Query,
            path: Vec::new(),
        }
    }

    pub fn is_root(&self) -> bool {
        self.path.is_empty()
    }

    #[must_use]
    pub fn push(&self, field: &'a str) -> GraphPath<'a> {
        let mut rv = self.clone();
        rv.path.push(field);
        rv
    }
}
