use std::{
    borrow::Cow,
    collections::{BTreeSet, HashMap, HashSet},
    iter::Peekable,
    marker::PhantomData,
};

use cynic_parser::{
    common::{TypeWrappers, WrappingType},
    type_system::{
        self as parser, ids::FieldDefinitionId, storage::InputValueDefinitionRecord, Definition,
        DirectiveDefinition, TypeDefinition,
    },
    Span,
};

use crate::schema::{names::FieldName, types::*, SchemaError};

#[ouroboros::self_referencing]
pub struct SchemaBackedTypeIndex {
    ast: cynic_parser::TypeSystemDocument,
    query_root: String,
    mutation_root: Option<String>,
    subscription_root: Option<String>,
    typename_field: FieldDefinitionId,
    #[borrows(ast)]
    #[covariant]
    types: HashMap<&'this str, TypeDefinition<'this>>,
    #[borrows(ast)]
    #[covariant]
    directives: HashMap<&'this str, DirectiveDefinition<'this>>,
}

impl SchemaBackedTypeIndex {
    pub fn for_schema(ast: cynic_parser::TypeSystemDocument) -> Self {
        let mut query_root = "Query".to_string();
        let mut mutation_root = None;
        let mut subscription_root = None;

        for definition in ast.definitions() {
            if let Definition::Schema(schema) = definition {
                if let Some(query_type) = schema.query_type() {
                    query_root = query_type.named_type().to_owned();
                }
                mutation_root = schema
                    .mutation_type()
                    .map(|mutation| mutation.named_type())
                    .map(ToOwned::to_owned);
                subscription_root = schema
                    .subscription_type()
                    .map(|subscription| subscription.named_type())
                    .map(ToOwned::to_owned);
                break;
            }
        }

        let (typename_field, ast) = add_builtins_to_document(ast);

        SchemaBackedTypeIndex::new(
            ast,
            query_root,
            mutation_root,
            subscription_root,
            typename_field,
            |ast| {
                let mut types = HashMap::new();
                for definition in ast.definitions() {
                    if let Definition::Type(type_def) = definition {
                        types.insert(type_def.name(), type_def);
                    }
                }
                types
            },
            |ast| {
                let mut directives = HashMap::new();
                for definition in ast.definitions() {
                    if let Definition::Directive(directive) = definition {
                        directives.insert(directive.name(), directive);
                    }
                }
                directives
            },
        )
    }
}

/// Adds the various builtins that might be omitted from the document
fn add_builtins_to_document(
    ast: parser::TypeSystemDocument,
) -> (FieldDefinitionId, parser::TypeSystemDocument) {
    let mut has_skip = false;
    let mut has_include = false;
    for directive in ast.directive_definitions() {
        if directive.name() == "skip" {
            has_skip = true;
        }
        if directive.name() == "include" {
            has_include = true;
        }
    }

    let mut writer = parser::writer::TypeSystemAstWriter::update(ast);
    let span = cynic_parser::Span::new(0, 0);
    for builtin in BUILTIN_SCALARS {
        let name = writer.ident(builtin);
        writer.scalar_definition(parser::storage::ScalarDefinitionRecord {
            name,
            name_span: Span::new(0, 0),
            description: None,
            directives: Default::default(),
            span,
        });
    }
    let typename_string = writer.ident("__typename");
    let string_ident = writer.ident("String");
    let typename_type = writer.type_reference(parser::storage::TypeRecord {
        name: string_ident,
        name_start: 0,
        span: Span::new(0, 0),
        wrappers: TypeWrappers::none().wrap_non_null(),
    });
    let typename_field = writer.field_definition(parser::storage::FieldDefinitionRecord {
        name: typename_string,
        name_span: Span::new(0, 0),
        ty: typename_type,
        arguments: Default::default(),
        description: None,
        directives: Default::default(),
        span,
    });

    if !has_skip || !has_include {
        let if_argument_type = parser::storage::TypeRecord {
            name: writer.ident("Boolean"),
            name_start: 0,
            span: Span::new(0, 0),
            wrappers: [WrappingType::NonNull].into_iter().collect(),
        };
        let if_argument_type = writer.type_reference(if_argument_type);
        let if_argument = InputValueDefinitionRecord {
            name: writer.ident("if"),
            name_span: Span::new(0, 0),
            ty: if_argument_type,
            description: None,
            default_value: None,
            default_value_span: Span::new(0, 0),
            directives: Default::default(),
            span,
        };

        writer.input_value_definition(if_argument);
        let arguments = writer.input_value_definition_range(Some(1));

        if !has_skip {
            let skip = parser::storage::DirectiveDefinitionRecord {
                name: writer.ident("skip"),
                name_span: Span::new(0, 0),
                description: None,
                arguments,
                is_repeatable: false,
                locations: vec![
                    parser::DirectiveLocation::Field,
                    parser::DirectiveLocation::FragmentSpread,
                    parser::DirectiveLocation::InlineFragment,
                ],
                span,
            };
            writer.directive_definition(skip);
        }

        if !has_include {
            let include = parser::storage::DirectiveDefinitionRecord {
                name: writer.ident("include"),
                name_span: Span::new(0, 0),
                description: None,
                arguments,
                is_repeatable: false,
                locations: vec![
                    parser::DirectiveLocation::Field,
                    parser::DirectiveLocation::FragmentSpread,
                    parser::DirectiveLocation::InlineFragment,
                ],
                span,
            };
            writer.directive_definition(include);
        }
    }

    let ast = writer.finish();
    (typename_field, ast)
}

impl super::TypeIndex for SchemaBackedTypeIndex {
    fn lookup_valid_type<'b>(&'b self, name: &str) -> Result<Type<'b>, SchemaError> {
        let type_def = self.borrow_types().get(name).copied().ok_or_else(|| {
            SchemaError::CouldNotFindType {
                name: name.to_string(),
            }
        })?;

        self.validate(vec![type_def])?;

        // Safe because we validated
        Ok(self.unsafe_lookup(name))
    }

    fn lookup_directive<'b>(&'b self, name: &str) -> Result<Option<Directive<'b>>, SchemaError> {
        let Some(directive) = self.borrow_directives().get(name) else {
            return Ok(None);
        };

        self.validate_directive(directive)?;

        // Safe because we validated
        Ok(self.unsafe_directive_lookup(name))
    }

    fn validate_all(&self) -> Result<(), SchemaError> {
        self.validate(self.borrow_types().values().copied().collect())?;

        for directive in self.borrow_directives().values() {
            self.validate_directive(directive)?;
        }

        Ok(())
    }

    fn root_types(&self) -> Result<SchemaRoots<'_>, SchemaError> {
        Ok(SchemaRoots {
            query: self
                .lookup_valid_type(self.borrow_query_root())?
                .try_into()?,
            mutation: self
                .borrow_mutation_root()
                .as_ref()
                .map(|name| ObjectType::try_from(self.lookup_valid_type(name)?))
                .transpose()?
                .or_else(|| ObjectType::try_from(self.lookup_valid_type("Mutation").ok()?).ok()),
            subscription: self
                .borrow_subscription_root()
                .as_ref()
                .map(|name| ObjectType::try_from(self.lookup_valid_type(name)?))
                .transpose()?
                .or_else(|| {
                    ObjectType::try_from(self.lookup_valid_type("Subscription").ok()?).ok()
                }),
        })
    }

    fn unsafe_lookup<'b>(&'b self, name: &str) -> Type<'b> {
        // Note: This function should absolutely only be called after the hierarchy has
        // been validated.  The current module privacy settings enforce this, but don't make this
        // private or call it without being careful.
        let type_def = self
            .borrow_types()
            .get(name)
            .copied()
            .expect("Couldn't find a type - this should be impossible");

        match type_def {
            TypeDefinition::Scalar(def) => Type::Scalar(ScalarType {
                name: Cow::Borrowed(def.name()),
                builtin: scalar_is_builtin(def.name()),
            }),
            TypeDefinition::Object(def) => {
                let mut fields = def
                    .fields()
                    .map(|field| build_field(field, def.name()))
                    .collect::<Vec<_>>();

                fields.push(build_field(
                    self.borrow_ast().read(*self.borrow_typename_field()),
                    def.name(),
                ));

                Type::Object(ObjectType {
                    name: Cow::Borrowed(def.name()),
                    fields,
                    implements_interfaces: def
                        .implements_interfaces()
                        .map(|iface| InterfaceRef(Cow::Borrowed(iface)))
                        .collect(),
                })
            }
            TypeDefinition::Interface(def) => {
                let mut fields = def
                    .fields()
                    .map(|field| build_field(field, def.name()))
                    .collect::<Vec<_>>();

                fields.push(build_field(
                    self.borrow_ast().read(*self.borrow_typename_field()),
                    def.name(),
                ));

                Type::Interface(InterfaceType {
                    name: Cow::Borrowed(def.name()),
                    fields,
                })
            }
            TypeDefinition::Union(def) => Type::Union(UnionType {
                name: Cow::Borrowed(def.name()),
                types: def
                    .members()
                    .map(|member| member.name())
                    .map(|name| ObjectRef(Cow::Borrowed(name)))
                    .collect(),
            }),
            TypeDefinition::Enum(def) => Type::Enum(EnumType {
                name: Cow::Borrowed(def.name()),
                values: def
                    .values()
                    .map(|val| EnumValue {
                        name: FieldName::new(val.value()),
                    })
                    .collect(),
            }),
            TypeDefinition::InputObject(def) => Type::InputObject(InputObjectType {
                name: Cow::Borrowed(def.name()),
                fields: def.fields().map(convert_input_value).collect(),
            }),
        }
    }

    fn unsafe_iter<'b>(&'b self) -> Box<dyn Iterator<Item = Type<'b>> + 'b> {
        let keys = self.borrow_types().keys().collect::<BTreeSet<_>>();

        Box::new(keys.into_iter().map(|name| self.unsafe_lookup(name)))
    }

    fn unsafe_directive_lookup<'b>(&'b self, name: &str) -> Option<Directive<'b>> {
        let parser_directive = self.borrow_directives().get(name)?;

        Some(Directive {
            name: Cow::Borrowed(parser_directive.name()),
            arguments: parser_directive
                .arguments()
                .map(convert_input_value)
                .collect(),
            locations: parser_directive.locations().map(Into::into).collect(),
        })
    }

    fn unsafe_directive_iter<'a>(&'a self) -> Box<dyn Iterator<Item = Directive<'a>> + 'a> {
        let keys = self.borrow_directives().keys().collect::<BTreeSet<_>>();

        Box::new(
            keys.into_iter()
                .map(|name| self.unsafe_directive_lookup(name).unwrap()),
        )
    }
}

impl SchemaBackedTypeIndex {
    fn lookup_type<'a>(&'a self, name: &str) -> Option<TypeDefinition<'a>> {
        self.borrow_types().get(name).copied()
    }

    /// Validates that all the types contained within the given types do exist.
    ///
    /// So we can just directly use refs to them.
    fn validate<'a>(&'a self, mut defs: Vec<TypeDefinition<'a>>) -> Result<(), SchemaError> {
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
                    for field in obj.fields() {
                        let name = field.ty().name();
                        validate!(name, Input);
                    }
                }
                TypeDefinition::Scalar(_) => {}
                TypeDefinition::Enum(_) => {}
                TypeDefinition::Object(obj) => {
                    for field in obj.fields() {
                        let name = field.ty().name();
                        validate!(name, Output);
                        for field in field.arguments() {
                            let name = field.ty().name();
                            validate!(name, Input);
                        }
                    }
                    for name in obj.implements_interfaces() {
                        validate!(
                            name,
                            TypeDefinition::Interface(_),
                            "expected to be an interface"
                        );
                    }
                }
                TypeDefinition::Union(union_def) => {
                    for member in union_def.members() {
                        let name = member.name();
                        validate!(name, TypeDefinition::Object(_), "expected to be an object");
                    }
                }
                TypeDefinition::Interface(iface) => {
                    for field in iface.fields() {
                        let name = field.ty().name();
                        validate!(name, Output);
                        for field in field.arguments() {
                            let name = field.ty().name();
                            validate!(name, Input);
                        }
                    }
                    for name in iface.implements_interfaces() {
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

    fn validate_directive(&self, directive: &DirectiveDefinition<'_>) -> Result<(), SchemaError> {
        let mut definitions = vec![];
        for argument in directive.arguments() {
            let named_type = argument.ty().name();
            let def = self.lookup_type(named_type);
            let Some(ty) = def else {
                return Err(SchemaError::CouldNotFindType {
                    name: named_type.to_string(),
                });
            };
            definitions.push(ty);

            if !matches!(
                ty,
                TypeDefinition::InputObject(_)
                    | TypeDefinition::Enum(_)
                    | TypeDefinition::Scalar(_)
            ) {
                return Err(SchemaError::InvalidDirectiveArgument {
                    directive_name: directive.name().to_string(),
                    argument_name: argument.name().to_string(),
                    expected: Kind::InputType,
                    found: Kind::of_definition(ty),
                });
            }
        }
        self.validate(definitions)?;
        Ok(())
    }
}

static BUILTIN_SCALARS: [&str; 5] = ["String", "ID", "Int", "Float", "Boolean"];

fn scalar_is_builtin(name: &str) -> bool {
    BUILTIN_SCALARS.iter().any(|builtin| name == *builtin)
}

fn convert_input_value(val: cynic_parser::type_system::InputValueDefinition<'_>) -> InputValue<'_> {
    InputValue {
        name: FieldName {
            graphql_name: Cow::Borrowed(val.name()),
        },
        value_type: build_type_ref::<InputType<'_>>(val.ty()),
        has_default: val.default_value().is_some(),
    }
}

fn build_type_ref<T>(ty: parser::Type<'_>) -> TypeRef<'_, T> {
    fn inner_fn<T>(
        mut wrappers: Peekable<impl Iterator<Item = WrappingType>>,
        name: &str,
        nullable: bool,
    ) -> TypeRef<'_, T> {
        let next = wrappers.peek().copied();
        if next == Some(WrappingType::NonNull) {
            wrappers.next();
            return inner_fn::<T>(wrappers, name, false);
        }

        if nullable {
            return TypeRef::<T>::Nullable(Box::new(inner_fn::<T>(wrappers, name, false)));
        }

        match wrappers.next() {
            None => TypeRef::<T>::Named(Cow::Borrowed(name), PhantomData),
            Some(WrappingType::List) => {
                TypeRef::<T>::List(Box::new(inner_fn::<T>(wrappers, name, true)))
            }
            _ => panic!("This should be impossible"),
        }
    }

    inner_fn::<T>(ty.wrappers().peekable(), ty.name(), true)
}

fn build_field<'a>(field: parser::FieldDefinition<'a>, parent_type_name: &'a str) -> Field<'a> {
    Field {
        name: FieldName {
            graphql_name: Cow::Borrowed(field.name()),
        },
        arguments: field.arguments().map(convert_input_value).collect(),
        field_type: build_type_ref::<OutputType<'_>>(field.ty()),
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
        let ast = cynic_parser::parse_type_system_document(&schema).unwrap();
        let index = SchemaBackedTypeIndex::for_schema(ast);
        index.validate_all().unwrap();
    }

    #[test]
    fn test_build_type_ref_non_null_type() {
        let ast = ast_for_type("User!");
        let non_null_type = extract_type(&ast);

        assert_matches!(
            build_type_ref::<InputType<'_>>(non_null_type),
            TypeRef::Named(name, _) => {
                assert_eq!(name, "User");
            }
        );
    }

    #[test]
    fn test_build_type_ref_null_type() {
        let ast = ast_for_type("User");
        let nullable_type = extract_type(&ast);

        assert_matches!(
            build_type_ref::<InputType<'_>>(nullable_type),
            TypeRef::Nullable(inner) => {
                assert_matches!(*inner, TypeRef::Named(name, _) => {
                    assert_eq!(name, "User")
                })
            }
        );
    }

    #[test]
    fn test_build_type_ref_required_list_type() {
        let ast = ast_for_type("[User!]!");
        let required_list = extract_type(&ast);

        assert_matches!(
            build_type_ref::<InputType<'_>>(required_list),
            TypeRef::List(inner) => {
                assert_matches!(*inner, TypeRef::Named(name, _) => {
                    assert_eq!(name, "User")
                })
            }
        );
    }

    #[test]
    fn test_build_type_ref_option_list_type() {
        let ast = ast_for_type("[User]");
        let optional_list = extract_type(&ast);

        assert_matches!(
            build_type_ref::<InputType<'_>>(optional_list),
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

    #[test]
    fn test_build_type_ref_option_list_of_required() {
        let ast = ast_for_type("[User!]");
        let optional_list = extract_type(&ast);

        assert_matches!(
            build_type_ref::<InputType<'_>>(optional_list),
            TypeRef::Nullable(inner) => {
                assert_matches!(*inner, TypeRef::List(inner) => {
                    assert_matches!(*inner, TypeRef::Named(name, _) => {
                        assert_eq!(name, "User")
                    })
                })
            }
        );
    }

    fn ast_for_type(sdl_ty: &str) -> cynic_parser::TypeSystemDocument {
        cynic_parser::parse_type_system_document(&format!("type Blah {{ foo: {sdl_ty} }}")).unwrap()
    }

    fn extract_type(ast: &cynic_parser::TypeSystemDocument) -> parser::Type<'_> {
        let parser::Definition::Type(parser::TypeDefinition::Object(obj)) =
            ast.definitions().next().unwrap()
        else {
            panic!("something went wrong");
        };

        obj.fields().next().unwrap().ty()
    }
}
