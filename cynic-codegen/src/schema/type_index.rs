use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
    rc::Rc,
};

use once_cell::sync::Lazy;

use super::{names::FieldName, types::*, SchemaError};
use crate::schema::{self, parser, Definition, Document, TypeDefinition, TypeExt};

#[derive(Clone)]
pub struct TypeIndex<'a> {
    //name_to_kind: HashMap<String, Kind>,
    pub(super) types: Rc<HashMap<&'a str, &'a TypeDefinition>>,
}

impl<'a> TypeIndex<'a> {
    pub fn empty() -> Self {
        TypeIndex {
            types: Rc::new(HashMap::new()),
        }
    }

    pub(super) fn for_schema_2(document: &'a Document) -> Self {
        let mut types = HashMap::new();
        for definition in &document.definitions {
            if let Definition::TypeDefinition(type_def) = definition {
                types.insert(name_for_type(type_def), type_def);
            }
        }
        for def in BUILTIN_SCALARS.as_ref() {
            types.insert(name_for_type(def), def);
        }

        TypeIndex {
            types: Rc::new(types),
        }
    }

    #[deprecated]
    pub fn for_schema(document: &'a Document) -> Self {
        let mut types = HashMap::new();
        for definition in &document.definitions {
            if let Definition::TypeDefinition(type_def) = definition {
                types.insert(name_for_type(type_def), type_def);
            }
        }

        TypeIndex {
            types: Rc::new(types),
        }
    }

    pub fn lookup_valid_type(&self, name: &str) -> Result<Type<'a>, SchemaError> {
        let type_def =
            self.types
                .get(name)
                .copied()
                .ok_or_else(|| SchemaError::CouldNotFindType {
                    name: name.to_string(),
                })?;

        self.validate(vec![type_def])?;

        Ok(self.private_lookup(name).unwrap())
    }

    pub(super) fn validate_all(&self) -> Result<(), SchemaError> {
        self.validate(self.types.values().copied().collect())
    }

    pub(super) fn private_lookup(&self, name: &str) -> Option<Type<'a>> {
        // Note: This function should absolutely only be called after the heirarchy has
        // been validated.  The current module privacy settings enforce this, but don't make this
        // private or call it without being careful.
        let type_def = self
            .types
            .get(name)
            .copied()
            .expect("Couldn't find a type - this should be impossible");

        Some(match type_def {
            TypeDefinition::Scalar(def) => Type::Scalar(ScalarType {
                name: &def.name,
                builtin: scalar_is_builtin(&def.name),
            }),
            TypeDefinition::Object(def) => Type::Object(ObjectType {
                description: def.description.as_deref(),
                name: &def.name,
                fields: def
                    .fields
                    .iter()
                    .map(|field| Field {
                        description: field.description.as_deref(),
                        name: FieldName {
                            graphql_name: &field.name,
                        },
                        arguments: field
                            .arguments
                            .iter()
                            .map(|arg| convert_input_value(self, arg))
                            .collect(),
                        field_type: build_type_ref::<OutputType>(&field.field_type, self),
                        parent_type_name: &def.name,
                    })
                    .collect(),
                implements_interfaces: def
                    .implements_interfaces
                    .iter()
                    .map(|iface| InterfaceRef(iface.as_ref(), self.clone()))
                    .collect(),
            }),
            TypeDefinition::Interface(_) => todo!("iface lookup not done"),
            TypeDefinition::Union(def) => Type::Union(UnionType {
                description: def.description.as_deref(),
                name: &def.name,
                types: def
                    .types
                    .iter()
                    .map(|name| ObjectRef(name.as_str(), self.clone()))
                    .collect(),
            }),
            TypeDefinition::Enum(def) => Type::Enum(EnumType {
                description: def.description.as_deref(),
                name: &def.name,
                values: def
                    .values
                    .iter()
                    .map(|val| EnumValue {
                        description: val.description.as_deref(),
                        name: &val.name,
                    })
                    .collect(),
            }),
            TypeDefinition::InputObject(def) => Type::InputObject(InputObjectType {
                description: def.description.as_deref(),
                name: &def.name,
                fields: def
                    .fields
                    .iter()
                    .map(|field| convert_input_value(self, field))
                    .collect(),
            }),
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
    fn validate(&self, mut defs: Vec<&'a TypeDefinition>) -> Result<(), SchemaError> {
        let mut validated = HashSet::<&str>::new();

        macro_rules! validate {
            ($name:ident, Input) => {
                validate!(
                    $name,
                    TypeDefinition::InputObject(_)
                        | TypeDefinition::Enum(_)
                        | TypeDefinition::Scalar(_),
                    "expected to be an input type"
                );
            };
            ($name:ident, Output) => {
                validate!(
                    $name,
                    TypeDefinition::Object(_)
                        | TypeDefinition::Enum(_)
                        | TypeDefinition::Scalar(_)
                        | TypeDefinition::Union(_)
                        | TypeDefinition::Interface(_),
                    "expected to be an output type"
                );
            };
            ($name:ident, $is:pat, $err:literal) => {
                #[allow(deprecated)]
                let def = self.lookup_type($name);
                if !matches!(def, Some($is)) {
                    // TODO: Err
                    return Err(SchemaError::InvalidTypeInSchema {
                        name: $name.to_string(),
                        details: $err.to_string(),
                    });
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
                        validate!(
                            name,
                            TypeDefinition::Interface(_),
                            "expected to be an interface"
                        );
                    }
                }
                TypeDefinition::Union(union_def) => {
                    for member in &union_def.types {
                        let name = member.as_ref();
                        validate!(name, TypeDefinition::Object(_), "expected to be an object");
                    }
                }
                TypeDefinition::Interface(_) => {
                    todo!("validating interface")
                }
            }
        }

        Ok(())
    }
}

static BUILTIN_SCALARS: Lazy<[TypeDefinition; 5]> = Lazy::new(|| {
    [
        TypeDefinition::Scalar(parser::ScalarType {
            position: graphql_parser::Pos { line: 0, column: 0 },
            description: None,
            name: "String".to_string(),
            directives: Vec::new(),
        }),
        TypeDefinition::Scalar(parser::ScalarType {
            position: graphql_parser::Pos { line: 0, column: 0 },
            description: None,
            name: "ID".to_string(),
            directives: Vec::new(),
        }),
        TypeDefinition::Scalar(parser::ScalarType {
            position: graphql_parser::Pos { line: 0, column: 0 },
            description: None,
            name: "Int".to_string(),
            directives: Vec::new(),
        }),
        TypeDefinition::Scalar(parser::ScalarType {
            position: graphql_parser::Pos { line: 0, column: 0 },
            description: None,
            name: "Float".to_string(),
            directives: Vec::new(),
        }),
        TypeDefinition::Scalar(parser::ScalarType {
            position: graphql_parser::Pos { line: 0, column: 0 },
            description: None,
            name: "Boolean".to_string(),
            directives: Vec::new(),
        }),
    ]
});

fn scalar_is_builtin(name: &str) -> bool {
    BUILTIN_SCALARS
        .iter()
        .any(|s| matches!(s, TypeDefinition::Scalar(s) if s.name == name))
}

impl<'a, T> super::types::TypeRef<'a, T>
where
    Type<'a>: TryInto<T>,
    <Type<'a> as TryInto<T>>::Error: std::fmt::Debug,
    // T: 'a,
{
    pub fn inner_type(&self) -> T {
        match self {
            TypeRef::Named(name, index, _) => {
                // Note: we assume that TypeRef is only constructed after validation,
                // which makes this unwrap ok.
                // Probably want to enforce this via module heirarchy.
                index.private_lookup(name).unwrap().try_into().unwrap()
            }
            TypeRef::List(inner) => inner.inner_type(),
            TypeRef::Nullable(inner) => inner.inner_type(),
        }
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
    type_index: &TypeIndex<'a>,
    val: &'a schema::InputValue,
) -> InputValue<'a> {
    InputValue {
        description: val.description.as_deref(),
        name: FieldName {
            graphql_name: &val.name,
        },
        value_type: build_type_ref::<InputType>(&val.value_type, type_index),
    }
}

// TODO: Definitely test this fucker.
fn build_type_ref<'a, T>(ty: &'a schema::Type, type_index: &TypeIndex<'a>) -> TypeRef<'a, T> {
    fn inner_fn<'a, T>(
        ty: &'a schema::Type,
        type_index: &TypeIndex<'a>,
        nullable: bool,
    ) -> TypeRef<'a, T> {
        if let schema::Type::NonNullType(inner) = ty {
            return inner_fn::<T>(inner, type_index, false);
        }

        if nullable {
            return TypeRef::<T>::Nullable(Box::new(inner_fn::<T>(ty, type_index, false)));
        }

        match ty {
            schema::Type::NamedType(name) => {
                TypeRef::<T>::Named(name, type_index.to_owned(), PhantomData)
            }
            schema::Type::ListType(inner) => {
                TypeRef::<T>::List(Box::new(inner_fn::<T>(inner, type_index, true)))
            }
            _ => panic!("This should be impossible"),
        }
    }
    inner_fn::<T>(ty, type_index, true)
}

// TODO: deffo need tests of schema validation
