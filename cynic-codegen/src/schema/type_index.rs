use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
};

use super::types::*;
use crate::schema::{self, Definition, Document, TypeDefinition, TypeExt};

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
            if let Definition::TypeDefinition(type_def) = definition {
                types.insert(name_for_type(type_def), type_def);
            }
        }

        TypeIndex { types }
    }

    pub fn lookup_valid_type(&'_ self, name: &str) -> Result<Type<'_>, ()> {
        let type_def = self.types.get(name).copied().unwrap_or_else(|| todo!());
        self.validate(type_def)?;

        Ok(match type_def {
            TypeDefinition::Scalar(def) => Type::Scalar(ScalarType { name: &def.name }),
            TypeDefinition::Object(def) => Type::Object(ObjectType {
                description: def.description.as_deref(),
                name: &def.name,
                fields: def
                    .fields
                    .iter()
                    .map(|field| Field {
                        description: field.description.as_deref(),
                        name: &field.name,
                        arguments: field
                            .arguments
                            .iter()
                            .map(|arg| convert_input_value(self, arg))
                            .collect(),
                        field_type: build_type_ref::<OutputType>(&field.field_type, self),
                    })
                    .collect(),
            }),
            TypeDefinition::Interface(_) => todo!(),
            TypeDefinition::Union(_) => todo!(),
            TypeDefinition::Enum(_) => todo!(),
            TypeDefinition::InputObject(_) => todo!(),
        })
    }

    fn lookup_type(&self, name: &str) -> Option<&'a TypeDefinition> {
        #[allow(clippy::map_clone)]
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

    /// Validates that all the types contained within the given types do exist.
    ///
    /// So we can just directly use refs to them.
    fn validate(&self, def: &'a TypeDefinition) -> Result<(), ()> {
        let mut validated = HashSet::<&str>::new();
        let mut defs = vec![def];

        macro_rules! validate {
            ($name:ident, Input) => {
                validate!(
                    $name,
                    TypeDefinition::InputObject(_)
                        | TypeDefinition::Enum(_)
                        | TypeDefinition::Scalar(_)
                );
            };
            ($name:ident, Output) => {
                validate!(
                    $name,
                    TypeDefinition::Object(_)
                        | TypeDefinition::Enum(_)
                        | TypeDefinition::Scalar(_)
                        | TypeDefinition::Union(_)
                        | TypeDefinition::Interface(_)
                );
            };
            ($name:ident, $is:pat) => {
                #[allow(deprecated)]
                let def = self.lookup_type($name);
                if !matches!(def, Some($is)) {
                    // TODO: Err
                    return Err(());
                }
                if !validated.contains($name) {
                    validated.insert($name);
                    defs.push(def.unwrap());
                }
            };
        }

        while let Some(def) = defs.pop() {
            match def {
                TypeDefinition::InputObject(obj) => {
                    for field in &obj.fields {
                        let name = field.value_type.inner_name();
                        validate!(name, Input);
                    }
                }
                TypeDefinition::Scalar(_) => {}
                TypeDefinition::Enum(_) => {}
                TypeDefinition::Object(obj) => {
                    for field in &obj.fields {
                        let name = field.field_type.inner_name();
                        validate!(name, Output);
                        for field in &field.arguments {
                            let name = field.value_type.inner_name();
                            validate!(name, Input);
                        }
                    }
                    for iface in &obj.implements_interfaces {
                        let name = iface.as_ref();
                        validate!(name, TypeDefinition::Interface(_));
                    }
                }
                TypeDefinition::Union(_) => {}
                TypeDefinition::Interface(_) => {
                    todo!()
                }
            }
        }

        Ok(())
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

// Todo; move these ValidTypes somewhere better - maybe
// schema/type_index.rs
// schema/validated.rs
// schema/parser.rs

fn convert_input_value<'a>(
    type_index: &'a TypeIndex<'a>,
    val: &'a schema::InputValue,
) -> InputValue<'a> {
    InputValue {
        description: val.description.as_deref(),
        name: &val.name,
        value_type: build_type_ref::<InputType>(&val.value_type, type_index),
    }
}

// TODO: Definitely test this fucker.
fn build_type_ref<'a, T>(ty: &'a schema::Type, type_index: &'a TypeIndex) -> TypeRef<'a, T> {
    fn inner_fn<'a, T>(
        ty: &'a schema::Type,
        type_index: &'a TypeIndex,
        nullable: bool,
    ) -> TypeRef<'a, T> {
        if let schema::Type::NonNullType(inner) = ty {
            return inner_fn::<T>(inner, type_index, false);
        }

        if nullable {
            return TypeRef::<T>::Nullable(Box::new(inner_fn::<T>(ty, type_index, false)));
        }

        match ty {
            schema::Type::NamedType(name) => TypeRef::<T>::Named(name, type_index, PhantomData),
            schema::Type::ListType(inner) => {
                TypeRef::<T>::List(Box::new(inner_fn::<T>(inner, type_index, true)))
            }
            _ => panic!("This should be impossible"),
        }
    }
    inner_fn::<T>(ty, type_index, true)
}
