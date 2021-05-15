use cynic_parser::ast::{FragmentDef, Name};
use graphql_parser::schema::{Definition, ScalarType};
use std::{borrow::Cow, collections::HashMap, rc::Rc};

use crate::{
    schema::{Document, Field, TypeDefinition},
    type_ext::TypeExt,
    Error,
};

use super::{OutputField, Type};

pub struct TypeIndex<'schema> {
    types: HashMap<&'schema str, TypeDefinition<'schema>>,
    query_root: String,
    mutation_root: String,
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
            }
        }

        rv
    }

    pub fn field_for_path(
        self: &Rc<TypeIndex<'schema>>,
        path: GraphPath,
    ) -> Result<OutputField<'schema>, Error> {
        let root_name = match &path.operation_type {
            OperationType::Query => self.query_root.clone(),
            OperationType::Mutation => self.mutation_root.clone(),
            OperationType::Fragment(name) => name.text().to_string(),
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

        Ok(OutputField::from_parser(field, &self))
    }

    // Looks up the name of the type at Path.
    pub fn type_name_for_path(
        self: &Rc<Self>,
        path: GraphPath,
    ) -> Result<Cow<'schema, str>, Error> {
        match (path.is_root(), &path.operation_type) {
            (true, OperationType::Query) => Ok(Cow::Owned(self.query_root.clone())),
            (true, OperationType::Mutation) => Ok(Cow::Owned(self.mutation_root.clone())),
            (true, OperationType::Fragment(name)) => Ok(Cow::Owned(name.to_string())),
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

        Ok(Type::from_type_defintion(type_def, &self))
    }

    pub fn type_for_path(self: &Rc<Self>, path: GraphPath) -> Result<Type<'schema>, Error> {
        let type_name = self.type_name_for_path(path)?;
        self.lookup_type(type_name.as_ref())
    }

    fn find_field_recursive<'find>(
        &'find self,
        fields: &'find [Field<'schema>],
        current_type_name: &str,
        path: &[Name],
    ) -> Result<&'find Field<'schema>, Error> {
        match path {
            [] => panic!("This shouldn't happen"),
            [first] => fields
                .iter()
                .find(|field| field.name == first.text())
                .ok_or_else(|| {
                    Error::UnknownField(first.to_string(), current_type_name.to_string())
                }),
            [first, rest @ ..] => {
                let inner_name = fields
                    .iter()
                    .find(|field| field.name == first.text())
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
    Fragment(Name),
}

/// The path to a type within a graphql graph.
#[derive(Debug, PartialEq, Clone)]
pub struct GraphPath {
    operation_type: OperationType,
    path: Vec<Name>,
}

impl GraphPath {
    pub fn from_query_node(node: &impl cynic_parser::ast::AstNode) -> Self {
        use cynic_parser::ast::{AstNode, FieldSelection, NameOwner, OperationDef};

        let mut path = vec![];
        let mut operation_type = OperationType::Query;

        for node in node.syntax().ancestors() {
            if let Some(selection) = FieldSelection::cast(node.clone()) {
                path.extend(selection.name());
            }
            if let Some(op) = OperationDef::cast(node.clone()) {
                // TODO: a better API in the AST to do this would be nice
                if op
                    .operation_type()
                    .and_then(|o| o.mutation_keyword_token())
                    .is_some()
                {
                    operation_type = OperationType::Mutation;
                }
                // TODO: subscription support
                break;
            }
            if let Some(fragment) = FragmentDef::cast(node) {
                if let Some(name) = fragment
                    .type_condition()
                    .and_then(|tc| tc.named_type()?.name())
                {
                    operation_type = OperationType::Fragment(name)
                }
                break;
            }
        }
        // TODO: Clean up the names etc. in this file.
        path.reverse();
        GraphPath {
            operation_type,
            path,
        }
    }

    pub fn is_root(&self) -> bool {
        self.path.is_empty()
    }
}

// TODO: Test GraphPath::from_query_node above
