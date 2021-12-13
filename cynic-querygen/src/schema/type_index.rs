use graphql_parser::schema::{Definition, ScalarType};
use std::{borrow::Cow, collections::HashMap, rc::Rc};

use super::parser::{Document, Field, TypeDefinition};
use crate::{type_ext::TypeExt, Error};

use super::{OutputField, Type};

pub struct TypeIndex<'schema> {
    types: HashMap<&'schema str, TypeDefinition<'schema>>,
    query_root: String,
    mutation_root: String,
    subscription_root: String,
}

impl<'schema> TypeIndex<'schema> {
    pub fn from_schema(schema: &'_ Document<'schema>) -> TypeIndex<'schema> {
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
                if let Some(subscription) = schema_def.subscription {
                    rv.subscription_root = subscription.to_string();
                }
            }
        }

        rv
    }

    pub fn field_for_path<'path>(
        self: &Rc<TypeIndex<'schema>>,
        path: &GraphPath<'path>,
    ) -> Result<OutputField<'schema>, Error> {
        let root_name = match path.path_base {
            GraphPathBase::Query => self.query_root.clone(),
            GraphPathBase::Mutation => self.mutation_root.clone(),
            GraphPathBase::Subscription => self.subscription_root.clone(),
            GraphPathBase::Absolute(base) => base.to_string(),
        };

        let root = self
            .types
            .get(root_name.as_str())
            .ok_or_else(|| Error::CouldntFindRootType(root_name.clone()))?;

        let field = if let TypeDefinition::Object(root_object) = root {
            self.find_field_recursive(&root_object.fields, root_name.as_str(), &path.path)?
        } else {
            return Err(Error::ExpectedObject(root_name.to_string()));
        };

        Ok(OutputField::from_parser(field, self))
    }

    // Looks up the name of the type at Path.
    pub fn type_name_for_path<'path>(
        self: &Rc<Self>,
        path: &GraphPath<'path>,
    ) -> Result<Cow<'schema, str>, Error> {
        match (path.has_components(), &path.path_base) {
            (true, GraphPathBase::Query) => Ok(Cow::Owned(self.query_root.clone())),
            (true, GraphPathBase::Mutation) => Ok(Cow::Owned(self.mutation_root.clone())),
            (true, GraphPathBase::Subscription) => Ok(Cow::Owned(self.subscription_root.clone())),
            (true, GraphPathBase::Absolute(base)) => Ok(Cow::Owned(base.to_string())),
            (false, _) => Ok(Cow::Owned(
                self.field_for_path(path)?
                    .value_type
                    .inner_name()
                    .to_string(),
            )),
        }
    }

    pub fn lookup_type(self: &Rc<Self>, name: &str) -> Result<Type<'schema>, Error> {
        let type_def = self
            .types
            .get(name)
            .ok_or_else(|| Error::UnknownType(name.to_string()))?;

        Ok(Type::from_type_defintion(type_def, self))
    }

    pub fn type_for_path<'path>(
        self: &Rc<Self>,
        path: &GraphPath<'path>,
    ) -> Result<Type<'schema>, Error> {
        let type_name = self.type_name_for_path(path)?;
        self.lookup_type(type_name.as_ref())
    }

    fn find_field_recursive<'find, 'path>(
        &'find self,
        fields: &'find [Field<'schema>],
        current_type_name: &str,
        path: &[&'path str],
    ) -> Result<&'find Field<'schema>, Error> {
        match path {
            [] => panic!("This shouldn't happen"),
            [first] => fields
                .iter()
                .find(|field| field.name == *first)
                .ok_or_else(|| {
                    Error::UnknownField(first.to_string(), current_type_name.to_string())
                }),
            [first, rest @ ..] => {
                let inner_name = fields
                    .iter()
                    .find(|field| field.name == *first)
                    .ok_or_else(|| {
                        Error::UnknownField(first.to_string(), current_type_name.to_string())
                    })?
                    .field_type
                    .inner_name();

                let inner_type = self
                    .types
                    .get(inner_name)
                    .ok_or_else(|| Error::UnknownType(inner_name.to_string()))?;

                if let TypeDefinition::Object(object) = inner_type {
                    self.find_field_recursive(&object.fields, inner_name, rest)
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
        types.insert("Float", TypeDefinition::Scalar(ScalarType::new("Float")));
        types.insert(
            "Boolean",
            TypeDefinition::Scalar(ScalarType::new("Boolean")),
        );
        types.insert("ID", TypeDefinition::Scalar(ScalarType::new("ID")));

        TypeIndex {
            query_root: "Query".into(),
            mutation_root: "Mutation".into(),
            subscription_root: "Subscription".into(),
            types,
        }
    }
}

fn name_for_type<'a>(type_def: &TypeDefinition<'a>) -> &'a str {
    match type_def {
        TypeDefinition::Scalar(inner) => inner.name,
        TypeDefinition::Object(inner) => inner.name,
        TypeDefinition::Interface(inner) => inner.name,
        TypeDefinition::Union(inner) => inner.name,
        TypeDefinition::Enum(inner) => inner.name,
        TypeDefinition::InputObject(inner) => inner.name,
    }
}

#[derive(Debug, PartialEq, Clone)]
enum GraphPathBase<'a> {
    Query,
    Mutation,
    Subscription,
    Absolute(&'a str),
}

/// The path to a type within a graphql graph.
#[derive(Debug, PartialEq, Clone)]
pub struct GraphPath<'a> {
    path_base: GraphPathBase<'a>,
    path: Vec<&'a str>,
}

impl<'a> GraphPath<'a> {
    pub fn for_query() -> Self {
        GraphPath {
            path_base: GraphPathBase::Query,
            path: Vec::new(),
        }
    }

    pub fn for_mutation() -> Self {
        GraphPath {
            path_base: GraphPathBase::Mutation,
            path: Vec::new(),
        }
    }

    pub fn for_subscription() -> Self {
        GraphPath {
            path_base: GraphPathBase::Subscription,
            path: Vec::new(),
        }
    }

    pub fn for_named_type(name: &'a str) -> Self {
        GraphPath {
            path_base: GraphPathBase::Absolute(name),
            path: Vec::new(),
        }
    }

    fn has_components(&self) -> bool {
        self.path.is_empty()
    }

    #[must_use]
    pub fn push(&self, field: &'a str) -> GraphPath<'a> {
        let mut rv = self.clone();
        rv.path.push(field);
        rv
    }
}
