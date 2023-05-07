use std::{
    borrow::Cow,
    collections::{BTreeSet, HashMap, HashSet},
    marker::PhantomData,
    rc::Rc,
};

use once_cell::sync::Lazy;

use crate::schema::{
    names::FieldName,
    parser::{self, Definition, Document, TypeDefinition, TypeExt},
    types::*,
    SchemaError,
};

#[derive(Clone)]
pub struct SchemaBackedTypeIndex<'a> {
    // TODO: this should maybe just own the type defs?
    // might be easier...
    pub(super) types: Rc<HashMap<&'a str, &'a TypeDefinition>>,
}

impl<'a> SchemaBackedTypeIndex<'a> {
    #[cfg(test)]
    pub fn empty() -> Self {
        SchemaBackedTypeIndex {
            types: Rc::new(HashMap::new()),
        }
    }

    pub fn for_schema(document: &'a Document) -> Self {
        let mut types = HashMap::new();
        for definition in &document.definitions {
            if let Definition::TypeDefinition(type_def) = definition {
                types.insert(name_for_type(type_def), type_def);
            }
        }
        for def in BUILTIN_SCALARS.as_ref() {
            types.insert(name_for_type(def), def);
        }

        SchemaBackedTypeIndex {
            types: Rc::new(types),
        }
    }
}

impl<'a> super::TypeIndex for SchemaBackedTypeIndex<'a> {
    fn lookup_valid_type<'b>(&'b self, name: &str) -> Result<Type<'b>, SchemaError> {
        let type_def =
            self.types
                .get(name)
                .copied()
                .ok_or_else(|| SchemaError::CouldNotFindType {
                    name: name.to_string(),
                })?;

        self.validate(vec![type_def])?;

        // Safe because we validated
        Ok(self.unsafe_lookup(name).unwrap())
    }

    fn validate_all(&self) -> Result<(), SchemaError> {
        self.validate(self.types.values().copied().collect())
    }

    fn unsafe_lookup<'b>(&'b self, name: &str) -> Option<Type<'b>> {
        // Note: This function should absolutely only be called after the hierarchy has
        // been validated.  The current module privacy settings enforce this, but don't make this
        // private or call it without being careful.
        let type_def = self
            .types
            .get(name)
            .copied()
            .expect("Couldn't find a type - this should be impossible");

        Some(match type_def {
            TypeDefinition::Scalar(def) => Type::Scalar(ScalarType {
                name: Cow::Borrowed(&def.name),
                builtin: scalar_is_builtin(&def.name),
            }),
            TypeDefinition::Object(def) => Type::Object(ObjectType {
                name: Cow::Borrowed(&def.name),
                fields: def
                    .fields
                    .iter()
                    .map(|field| build_field(field, &def.name))
                    .collect(),
                implements_interfaces: def
                    .implements_interfaces
                    .iter()
                    .map(|iface| InterfaceRef(Cow::Borrowed(iface.as_ref())))
                    .collect(),
            }),
            TypeDefinition::Interface(def) => Type::Interface(InterfaceType {
                name: Cow::Borrowed(&def.name),
                fields: def
                    .fields
                    .iter()
                    .map(|f| build_field(f, &def.name))
                    .collect(),
            }),
            TypeDefinition::Union(def) => Type::Union(UnionType {
                name: Cow::Borrowed(&def.name),
                types: def
                    .types
                    .iter()
                    .map(|name| ObjectRef(Cow::Borrowed(name.as_str())))
                    .collect(),
            }),
            TypeDefinition::Enum(def) => Type::Enum(EnumType {
                name: Cow::Borrowed(&def.name),
                values: def
                    .values
                    .iter()
                    .map(|val| EnumValue {
                        name: FieldName::new(&val.name),
                    })
                    .collect(),
            }),
            TypeDefinition::InputObject(def) => Type::InputObject(InputObjectType {
                name: Cow::Borrowed(&def.name),
                fields: def.fields.iter().map(convert_input_value).collect(),
            }),
        })
    }

    fn unsafe_iter<'b>(&'b self) -> Box<dyn Iterator<Item = Type<'b>> + 'b> {
        let keys = self.types.keys().collect::<BTreeSet<_>>();

        Box::new(
            keys.into_iter()
                .map(|name| self.unsafe_lookup(name).unwrap()),
        )
    }
}

impl<'a> SchemaBackedTypeIndex<'a> {
    fn lookup_type(&self, name: &str) -> Option<&'a TypeDefinition> {
        #[allow(clippy::map_clone)]
        self.types.get(name).map(|d| *d)
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
                TypeDefinition::Interface(iface) => {
                    for field in &iface.fields {
                        let name = field.field_type.inner_name();
                        validate!(name, Output);
                        for field in &field.arguments {
                            let name = field.value_type.inner_name();
                            validate!(name, Input);
                        }
                    }
                    for iface in &iface.implements_interfaces {
                        let name = iface.as_ref();
                        validate!(
                            name,
                            TypeDefinition::Interface(_),
                            "expected to be an interface"
                        );
                    }
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

fn convert_input_value(val: &parser::InputValue) -> InputValue<'_> {
    InputValue {
        name: FieldName {
            graphql_name: Cow::Borrowed(&val.name),
        },
        value_type: build_type_ref::<InputType<'_>>(&val.value_type),
        has_default: val.default_value.is_some(),
    }
}

fn build_type_ref<'a, T>(ty: &'a parser::Type) -> TypeRef<'a, T> {
    fn inner_fn<'a, T>(ty: &'a parser::Type, nullable: bool) -> TypeRef<'a, T> {
        if let parser::Type::NonNullType(inner) = ty {
            return inner_fn::<T>(inner, false);
        }

        if nullable {
            return TypeRef::<T>::Nullable(Box::new(inner_fn::<T>(ty, false)));
        }

        match ty {
            parser::Type::NamedType(name) => TypeRef::<T>::Named(Cow::Borrowed(name), PhantomData),
            parser::Type::ListType(inner) => {
                TypeRef::<T>::List(Box::new(inner_fn::<T>(inner, true)))
            }
            _ => panic!("This should be impossible"),
        }
    }
    inner_fn::<T>(ty, true)
}

fn build_field<'a>(field: &'a parser::Field, parent_type_name: &'a str) -> Field<'a> {
    Field {
        name: FieldName {
            graphql_name: Cow::Borrowed(&field.name),
        },
        arguments: field
            .arguments
            .iter()
            .map(|arg| convert_input_value(arg))
            .collect(),
        field_type: build_type_ref::<OutputType<'_>>(&field.field_type),
        parent_type_name: Cow::Borrowed(parent_type_name),
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use rstest::rstest;
    use std::{fs, path::PathBuf};

    use super::*;
    use crate::schema::type_index::TypeIndex;

    #[rstest]
    #[case::starwars("starwars.schema.graphql")]
    #[case::github("github.graphql")]
    fn test_schema_validation_on_good_schemas(#[case] schema_file: &'static str) {
        let schema = fs::read_to_string(PathBuf::from("../schemas/").join(schema_file)).unwrap();
        let document = parser::parse_schema(&schema).unwrap();
        let index = SchemaBackedTypeIndex::for_schema(&document);
        index.validate_all().unwrap();
    }

    #[test]
    fn test_build_type_ref_non_null_type() {
        let index = &SchemaBackedTypeIndex::empty();
        let non_null_type =
            parser::Type::NonNullType(Box::new(parser::Type::NamedType("User".to_string())));

        assert_matches!(
            build_type_ref::<InputType<'_>>(&non_null_type),
            TypeRef::Named(name, _) => {
                assert_eq!(name, "User");
            }
        );
    }

    #[test]
    fn test_build_type_ref_null_type() {
        let nullable_type = parser::Type::NamedType("User".to_string());

        assert_matches!(
            build_type_ref::<InputType<'_>>(&nullable_type),
            TypeRef::Nullable(inner) => {
                assert_matches!(*inner, TypeRef::Named(name, _) => {
                    assert_eq!(name, "User")
                })
            }
        );
    }

    #[test]
    fn test_build_type_ref_required_list_type() {
        let required_list = parser::Type::NonNullType(Box::new(parser::Type::ListType(Box::new(
            parser::Type::NonNullType(Box::new(parser::Type::NamedType("User".to_string()))),
        ))));

        assert_matches!(
            build_type_ref::<InputType<'_>>(&required_list),
            TypeRef::List(inner) => {
                assert_matches!(*inner, TypeRef::Named(name, _) => {
                    assert_eq!(name, "User")
                })
            }
        );
    }

    #[test]
    fn test_build_type_ref_option_list_type() {
        let optional_list =
            parser::Type::ListType(Box::new(parser::Type::NamedType("User".to_string())));

        assert_matches!(
            build_type_ref::<InputType<'_>>(&optional_list),
            TypeRef::Nullable(inner) => {
                assert_matches!(*inner, TypeRef::List(inner) => {
                    assert_matches!(*inner, TypeRef::Nullable(inner) => {
                        assert_matches!(*inner, TypeRef::Named(name, _) => {
                            assert_eq!(name, "User")
                        })
                    })
                })
            }
        );
    }
}
