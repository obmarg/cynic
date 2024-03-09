use super::{ids::*, storage::*, AstLookup};
use super::{Ast, AstDefinition};

pub struct AstWriter {
    ast: Ast,
    field_id_cursor: FieldDefinitionId,
    input_value_id_cursor: InputValueDefinitionId,
    directive_id_cursor: DirectiveId,
}

// TODO: Don't forget the spans etc.
impl AstWriter {
    pub fn new() -> Self {
        AstWriter {
            ast: Default::default(),
            field_id_cursor: FieldDefinitionId::new(0),
            input_value_id_cursor: InputValueDefinitionId::new(0),
            directive_id_cursor: DirectiveId::new(0),
        }
    }

    pub fn update(ast: Ast) -> Self {
        AstWriter {
            field_id_cursor: FieldDefinitionId::new(ast.field_definitions.len()),
            input_value_id_cursor: InputValueDefinitionId::new(ast.input_value_definitions.len()),
            directive_id_cursor: DirectiveId::new(ast.directives.len()),
            ast,
        }
    }

    pub fn finish(self) -> Ast {
        // TODO: Possibly assert things in here for safety...
        self.ast
    }

    pub fn store_description(&mut self, definition: DefinitionId, description: Option<StringId>) {
        if let Some(description) = description {
            match *self.ast.lookup(definition) {
                AstDefinition::Schema(id) => {
                    self.ast.lookup_mut(id).description = Some(description);
                }
                AstDefinition::Scalar(id) => {
                    self.ast.lookup_mut(id).description = Some(description);
                }
                AstDefinition::Object(id) => {
                    self.ast.lookup_mut(id).description = Some(description);
                }
                AstDefinition::Interface(id) => {
                    self.ast.lookup_mut(id).description = Some(description);
                }
                AstDefinition::Union(id) => {
                    self.ast.lookup_mut(id).description = Some(description);
                }
                AstDefinition::Enum(id) => {
                    self.ast.lookup_mut(id).description = Some(description);
                }
                AstDefinition::InputObject(id) => {
                    self.ast.lookup_mut(id).description = Some(description);
                }
                AstDefinition::SchemaExtension(id) => {
                    self.ast.lookup_mut(id).description = Some(description);
                }
                AstDefinition::ScalarExtension(id) => {
                    self.ast.lookup_mut(id).description = Some(description);
                }
                AstDefinition::ObjectExtension(id) => {
                    self.ast.lookup_mut(id).description = Some(description);
                }
                AstDefinition::InterfaceExtension(id) => {
                    self.ast.lookup_mut(id).description = Some(description);
                }
                AstDefinition::UnionExtension(id) => {
                    self.ast.lookup_mut(id).description = Some(description);
                }
                AstDefinition::EnumExtension(id) => {
                    self.ast.lookup_mut(id).description = Some(description);
                }
                AstDefinition::InputObjectExtension(id) => {
                    self.ast.lookup_mut(id).description = Some(description);
                }
                AstDefinition::Directive(id) => {
                    self.ast.lookup_mut(id).description = Some(description);
                }
            }
        }
    }

    pub fn schema_definition(&mut self, definition: SchemaDefinition) -> DefinitionId {
        let id = SchemaDefinitionId::new(self.ast.schema_definitions.len());
        self.ast.schema_definitions.push(definition);

        let definition_id = DefinitionId::new(self.ast.definitions.len());
        self.ast.definitions.push(AstDefinition::Schema(id));

        definition_id
    }

    pub fn scalar_definition(&mut self, definition: ScalarDefinition) -> DefinitionId {
        let id = ScalarDefinitionId::new(self.ast.scalar_definitions.len());
        self.ast.scalar_definitions.push(definition);

        let definition_id = DefinitionId::new(self.ast.definitions.len());
        self.ast.definitions.push(AstDefinition::Scalar(id));

        definition_id
    }

    pub fn object_definition(&mut self, definition: ObjectDefinition) -> DefinitionId {
        // TODO: Maybe assert in here too?

        let id = ObjectDefinitionId::new(self.ast.object_definitions.len());
        self.ast.object_definitions.push(definition);

        let definition_id = DefinitionId::new(self.ast.definitions.len());
        self.ast.definitions.push(AstDefinition::Object(id));

        definition_id
    }

    pub fn field_definition(&mut self, definition: FieldDefinition) -> FieldDefinitionId {
        let definition_id = FieldDefinitionId::new(self.ast.field_definitions.len());
        self.ast.field_definitions.push(definition);

        definition_id
    }

    pub fn field_definition_range(
        &mut self,
        expected_count: Option<usize>,
    ) -> IdRange<FieldDefinitionId> {
        let start = self.field_id_cursor;
        let end = FieldDefinitionId::new(self.ast.field_definitions.len());
        self.field_id_cursor = end;
        let range = IdRange::new(start, end);

        if let Some(expected_count) = expected_count {
            assert_eq!(range.len(), expected_count);
        }

        range
    }

    pub fn interface_definition(&mut self, definition: InterfaceDefinition) -> DefinitionId {
        let id = InterfaceDefinitionId::new(self.ast.interface_definitions.len());
        self.ast.interface_definitions.push(definition);

        let definition_id = DefinitionId::new(self.ast.definitions.len());
        self.ast.definitions.push(AstDefinition::Interface(id));

        definition_id
    }

    pub fn union_definition(&mut self, definition: UnionDefinition) -> DefinitionId {
        let id = UnionDefinitionId::new(self.ast.union_definitions.len());
        self.ast.union_definitions.push(definition);

        let definition_id = DefinitionId::new(self.ast.definitions.len());
        self.ast.definitions.push(AstDefinition::Union(id));

        definition_id
    }

    pub fn enum_definition(&mut self, definition: EnumDefinition) -> DefinitionId {
        let id = EnumDefinitionId::new(self.ast.enum_definitions.len());
        self.ast.enum_definitions.push(definition);

        let definition_id = DefinitionId::new(self.ast.definitions.len());
        self.ast.definitions.push(AstDefinition::Enum(id));

        definition_id
    }

    pub fn enum_value_definition(
        &mut self,
        definition: EnumValueDefinition,
    ) -> EnumValueDefinitionId {
        let id = EnumValueDefinitionId::new(self.ast.enum_value_definitions.len());
        self.ast.enum_value_definitions.push(definition);

        id
    }

    pub fn input_object_definition(&mut self, definition: InputObjectDefinition) -> DefinitionId {
        let id = InputObjectDefinitionId::new(self.ast.input_object_definitions.len());
        self.ast.input_object_definitions.push(definition);

        let definition_id = DefinitionId::new(self.ast.definitions.len());
        self.ast.definitions.push(AstDefinition::InputObject(id));

        definition_id
    }

    pub fn input_value_definition(
        &mut self,
        definition: InputValueDefinition,
    ) -> InputValueDefinitionId {
        let definition_id = InputValueDefinitionId::new(self.ast.input_value_definitions.len());
        self.ast.input_value_definitions.push(definition);

        definition_id
    }

    pub fn input_value_definition_range(
        &mut self,
        expected_count: Option<usize>,
    ) -> IdRange<InputValueDefinitionId> {
        let start = self.input_value_id_cursor;
        let end = InputValueDefinitionId::new(self.ast.input_value_definitions.len());
        self.input_value_id_cursor = end;
        let range = IdRange::new(start, end);

        if let Some(expected_count) = expected_count {
            assert_eq!(range.len(), expected_count);
        }

        range
    }

    pub fn schema_extension(&mut self, definition: SchemaDefinition) -> DefinitionId {
        let id = SchemaDefinitionId::new(self.ast.schema_definitions.len());
        self.ast.schema_definitions.push(definition);

        let definition_id = DefinitionId::new(self.ast.definitions.len());
        self.ast
            .definitions
            .push(AstDefinition::SchemaExtension(id));

        definition_id
    }

    pub fn scalar_extension(&mut self, definition: ScalarDefinition) -> DefinitionId {
        let id = ScalarDefinitionId::new(self.ast.scalar_definitions.len());
        self.ast.scalar_definitions.push(definition);

        let definition_id = DefinitionId::new(self.ast.definitions.len());
        self.ast
            .definitions
            .push(AstDefinition::ScalarExtension(id));

        definition_id
    }

    pub fn object_extension(&mut self, definition: ObjectDefinition) -> DefinitionId {
        let id = ObjectDefinitionId::new(self.ast.object_definitions.len());
        self.ast.object_definitions.push(definition);

        let definition_id = DefinitionId::new(self.ast.definitions.len());
        self.ast
            .definitions
            .push(AstDefinition::ObjectExtension(id));

        definition_id
    }

    pub fn interface_extension(&mut self, definition: InterfaceDefinition) -> DefinitionId {
        let id = InterfaceDefinitionId::new(self.ast.interface_definitions.len());
        self.ast.interface_definitions.push(definition);

        let definition_id = DefinitionId::new(self.ast.definitions.len());
        self.ast
            .definitions
            .push(AstDefinition::InterfaceExtension(id));

        definition_id
    }

    pub fn union_extension(&mut self, definition: UnionDefinition) -> DefinitionId {
        let id = UnionDefinitionId::new(self.ast.union_definitions.len());
        self.ast.union_definitions.push(definition);

        let definition_id = DefinitionId::new(self.ast.definitions.len());
        self.ast.definitions.push(AstDefinition::UnionExtension(id));

        definition_id
    }

    pub fn enum_extension(&mut self, definition: EnumDefinition) -> DefinitionId {
        let id = EnumDefinitionId::new(self.ast.enum_definitions.len());
        self.ast.enum_definitions.push(definition);

        let definition_id = DefinitionId::new(self.ast.definitions.len());
        self.ast.definitions.push(AstDefinition::EnumExtension(id));

        definition_id
    }

    pub fn input_object_extension(&mut self, definition: InputObjectDefinition) -> DefinitionId {
        let id = InputObjectDefinitionId::new(self.ast.input_object_definitions.len());
        self.ast.input_object_definitions.push(definition);

        let definition_id = DefinitionId::new(self.ast.definitions.len());
        self.ast
            .definitions
            .push(AstDefinition::InputObjectExtension(id));

        definition_id
    }

    pub fn directive_definition(&mut self, definition: DirectiveDefinition) -> DefinitionId {
        let id = DirectiveDefinitionId::new(self.ast.directive_definitions.len());
        self.ast.directive_definitions.push(definition);

        let definition_id = DefinitionId::new(self.ast.definitions.len());
        self.ast.definitions.push(AstDefinition::Directive(id));

        definition_id
    }

    pub fn directive_range(&mut self, expected_count: Option<usize>) -> IdRange<DirectiveId> {
        let start = self.directive_id_cursor;
        let end = DirectiveId::new(self.ast.directives.len());
        self.directive_id_cursor = end;
        let range = IdRange::new(start, end);

        if let Some(expected_count) = expected_count {
            assert_eq!(range.len(), expected_count);
        }

        range
    }

    pub fn type_reference(&mut self, ty: Type) -> TypeId {
        let ty_id = TypeId::new(self.ast.type_references.len());
        self.ast.type_references.push(ty);
        ty_id
    }

    pub fn directive(&mut self, directive: Directive) -> DirectiveId {
        let id = DirectiveId::new(self.ast.directives.len());
        self.ast.directives.push(directive);
        id
    }

    pub fn argument(&mut self, argument: Argument) -> ArgumentId {
        let id = ArgumentId::new(self.ast.arguments.len());
        self.ast.arguments.push(argument);
        id
    }

    pub fn value(&mut self, value: Value) -> ValueId {
        let id = ValueId::new(self.ast.values.len());
        self.ast.values.push(value);
        id
    }

    pub fn string_literal(&mut self, literal: StringLiteral) -> StringLiteralId {
        let literal_id = StringLiteralId(self.ast.string_literals.len());
        self.ast.string_literals.push(literal);

        literal_id
    }

    pub fn ident(&mut self, ident: &str) -> StringId {
        self.intern_string(ident)
    }

    // TOOD: should this be pub? not sure...
    pub fn intern_string(&mut self, string: &str) -> StringId {
        let (id, _) = self.ast.strings.insert_full(string.into());
        StringId::new(id)
    }
}

impl Default for AstWriter {
    fn default() -> Self {
        Self::new()
    }
}
