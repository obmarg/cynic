use cynic_parser::{
    type_system::{
        self, ids::FieldDefinitionId, Definition, DirectiveDefinition, FieldDefinition,
        TypeDefinition,
    },
    TypeSystemDocument,
};
use std::{borrow::Cow, collections::HashMap, rc::Rc};

use crate::Error;

use super::{OutputField, Type};

pub struct TypeIndex<'schema> {
    types: HashMap<&'schema str, TypeDefinition<'schema>>,
    directives: HashMap<&'schema str, DirectiveDefinition<'schema>>,
    typename_field: FieldDefinition<'schema>,
    query_root: String,
    mutation_root: String,
    subscription_root: String,
}

impl<'schema> TypeIndex<'schema> {
    pub fn from_schema(
        schema: &'schema TypeSystemDocument,
        typename_id: FieldDefinitionId,
    ) -> TypeIndex<'schema> {
        let types = schema
            .definitions()
            .filter_map(|definition| match definition {
                Definition::Type(ty) => Some((ty.name(), ty)),
                // TODO: Someday need to support extensions here...
                _ => None,
            })
            .collect::<HashMap<_, _>>();

        let directives = schema
            .definitions()
            .filter_map(|definition| match definition {
                Definition::Directive(def) => Some((def.name(), def)),
                _ => None,
            })
            .collect::<HashMap<_, _>>();

        let mut rv = TypeIndex {
            query_root: "Query".into(),
            mutation_root: "Mutation".into(),
            subscription_root: "Subscription".into(),
            typename_field: schema.read(typename_id),
            types,
            directives,
        };

        for definition in schema.definitions() {
            if let Definition::Schema(schema_def) = definition {
                if let Some(query) = schema_def.query_type() {
                    rv.query_root = query.named_type().to_string();
                }
                if let Some(mutation) = schema_def.mutation_type() {
                    rv.mutation_root = mutation.named_type().to_string();
                }
                if let Some(subscription) = schema_def.subscription_type() {
                    rv.subscription_root = subscription.named_type().to_string();
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

        let field = match root {
            TypeDefinition::Object(object) => {
                self.find_field_recursive(object.fields(), root_name.as_str(), &path.path)?
            }
            TypeDefinition::Interface(iface) => {
                self.find_field_recursive(iface.fields(), root_name.as_str(), &path.path)?
            }
            _ => {
                return Err(Error::ExpectedObject(root_name.to_string()));
            }
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

        Ok(Type::from_type_definition(type_def, self))
    }

    pub fn directive(self: &Rc<Self>, name: &str) -> Result<DirectiveDefinition<'schema>, Error> {
        self.directives
            .get(name)
            .copied()
            .ok_or_else(|| Error::UnknownDirective(name.to_string()))
    }

    pub fn type_for_path<'path>(
        self: &Rc<Self>,
        path: &GraphPath<'path>,
    ) -> Result<Type<'schema>, Error> {
        let type_name = self.type_name_for_path(path)?;
        self.lookup_type(type_name.as_ref())
    }

    pub fn typename_field(&self) -> type_system::FieldDefinition<'schema> {
        self.typename_field
    }

    fn find_field_recursive(
        &self,
        mut fields: impl Iterator<Item = FieldDefinition<'schema>>,
        current_type_name: &str,
        path: &[&str],
    ) -> Result<FieldDefinition<'schema>, Error> {
        match path {
            [] => panic!("This shouldn't happen"),
            ["__typename"] => Ok(self.typename_field()),
            ["__typename", _rest @ ..] => {
                Err(Error::TriedToSelectFieldsOfNonComposite("String".into()))
            }
            [first] => fields.find(|field| field.name() == *first).ok_or_else(|| {
                Error::UnknownField(first.to_string(), current_type_name.to_string())
            }),
            [first, rest @ ..] => {
                let inner_name = fields
                    .find(|field| field.name() == *first)
                    .ok_or_else(|| {
                        Error::UnknownField(first.to_string(), current_type_name.to_string())
                    })?
                    .ty()
                    .name();

                let inner_type = self
                    .types
                    .get(inner_name)
                    .copied()
                    .ok_or_else(|| Error::UnknownType(inner_name.to_string()))?;

                match inner_type {
                    TypeDefinition::Object(object) => {
                        self.find_field_recursive(object.fields(), inner_name, rest)
                    }
                    TypeDefinition::Interface(iface) => {
                        self.find_field_recursive(iface.fields(), inner_name, rest)
                    }
                    _ => Err(Error::ExpectedObjectOrInterface(inner_name.to_string())),
                }
            }
        }
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
