use crate::common::IdRange;
use crate::AstLookup;

use super::{ids::*, storage::*};
use super::{DefinitionRecord, TypeSystemDocument};

pub struct TypeSystemAstWriter {
    ast: TypeSystemDocument,
    field_id_cursor: FieldDefinitionId,
    input_value_id_cursor: InputValueDefinitionId,
    enum_value_id_cursor: EnumValueDefinitionId,
    directive_id_cursor: DirectiveId,
    argument_id_cursor: ArgumentId,
    union_member_id_cursor: UnionMemberId,
}

// TODO: Don't forget the spans etc.
impl TypeSystemAstWriter {
    pub fn new() -> Self {
        TypeSystemAstWriter {
            ast: Default::default(),
            field_id_cursor: FieldDefinitionId::new(0),
            input_value_id_cursor: InputValueDefinitionId::new(0),
            enum_value_id_cursor: EnumValueDefinitionId::new(0),
            directive_id_cursor: DirectiveId::new(0),
            argument_id_cursor: ArgumentId::new(0),
            union_member_id_cursor: UnionMemberId::new(0),
        }
    }

    pub fn update(ast: TypeSystemDocument) -> Self {
        TypeSystemAstWriter {
            field_id_cursor: FieldDefinitionId::new(ast.field_definitions.len()),
            input_value_id_cursor: InputValueDefinitionId::new(ast.input_value_definitions.len()),
            directive_id_cursor: DirectiveId::new(ast.directives.len()),
            enum_value_id_cursor: EnumValueDefinitionId::new(ast.enum_value_definitions.len()),
            argument_id_cursor: ArgumentId::new(ast.arguments.len()),
            union_member_id_cursor: UnionMemberId::new(ast.union_members.len()),
            ast,
        }
    }

    pub fn finish(self) -> TypeSystemDocument {
        // TODO: Possibly assert things in here for safety...
        self.ast
    }

    pub fn store_description(
        &mut self,
        definition: DefinitionId,
        description: Option<StringLiteralId>,
    ) {
        if let Some(description) = description {
            match *self.ast.lookup(definition) {
                DefinitionRecord::Schema(id) => {
                    self.ast.lookup_mut(id).description = Some(description);
                }
                DefinitionRecord::Scalar(id) => {
                    self.ast.lookup_mut(id).description = Some(description);
                }
                DefinitionRecord::Object(id) => {
                    self.ast.lookup_mut(id).description = Some(description);
                }
                DefinitionRecord::Interface(id) => {
                    self.ast.lookup_mut(id).description = Some(description);
                }
                DefinitionRecord::Union(id) => {
                    self.ast.lookup_mut(id).description = Some(description);
                }
                DefinitionRecord::Enum(id) => {
                    self.ast.lookup_mut(id).description = Some(description);
                }
                DefinitionRecord::InputObject(id) => {
                    self.ast.lookup_mut(id).description = Some(description);
                }
                DefinitionRecord::SchemaExtension(id) => {
                    self.ast.lookup_mut(id).description = Some(description);
                }
                DefinitionRecord::ScalarExtension(id) => {
                    self.ast.lookup_mut(id).description = Some(description);
                }
                DefinitionRecord::ObjectExtension(id) => {
                    self.ast.lookup_mut(id).description = Some(description);
                }
                DefinitionRecord::InterfaceExtension(id) => {
                    self.ast.lookup_mut(id).description = Some(description);
                }
                DefinitionRecord::UnionExtension(id) => {
                    self.ast.lookup_mut(id).description = Some(description);
                }
                DefinitionRecord::EnumExtension(id) => {
                    self.ast.lookup_mut(id).description = Some(description);
                }
                DefinitionRecord::InputObjectExtension(id) => {
                    self.ast.lookup_mut(id).description = Some(description);
                }
                DefinitionRecord::Directive(id) => {
                    self.ast.lookup_mut(id).description = Some(description);
                }
            }
        }
    }

    pub fn schema_definition(&mut self, definition: SchemaDefinitionRecord) -> DefinitionId {
        let id = SchemaDefinitionId::new(self.ast.schema_definitions.len());
        self.ast.schema_definitions.push(definition);

        let definition_id = DefinitionId::new(self.ast.definitions.len());
        self.ast.definitions.push(DefinitionRecord::Schema(id));

        definition_id
    }

    pub fn root_operation_definitions(
        &mut self,
        definitions: Vec<RootOperationTypeDefinitionRecord>,
    ) -> IdRange<RootOperationTypeDefinitionId> {
        let start_id =
            RootOperationTypeDefinitionId::new(self.ast.root_operation_definitions.len());

        self.ast.root_operation_definitions.extend(definitions);
        let end_id = RootOperationTypeDefinitionId::new(self.ast.root_operation_definitions.len());

        IdRange::new(start_id, end_id)
    }

    pub fn scalar_definition(&mut self, definition: ScalarDefinitionRecord) -> DefinitionId {
        let id = ScalarDefinitionId::new(self.ast.scalar_definitions.len());
        self.ast.scalar_definitions.push(definition);

        let definition_id = DefinitionId::new(self.ast.definitions.len());
        self.ast.definitions.push(DefinitionRecord::Scalar(id));

        definition_id
    }

    pub fn object_definition(&mut self, definition: ObjectDefinitionRecord) -> DefinitionId {
        // TODO: Maybe assert in here too?

        let id = ObjectDefinitionId::new(self.ast.object_definitions.len());
        self.ast.object_definitions.push(definition);

        let definition_id = DefinitionId::new(self.ast.definitions.len());
        self.ast.definitions.push(DefinitionRecord::Object(id));

        definition_id
    }

    pub fn field_definition(&mut self, definition: FieldDefinitionRecord) -> FieldDefinitionId {
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

    pub fn interface_definition(&mut self, definition: InterfaceDefinitionRecord) -> DefinitionId {
        let id = InterfaceDefinitionId::new(self.ast.interface_definitions.len());
        self.ast.interface_definitions.push(definition);

        let definition_id = DefinitionId::new(self.ast.definitions.len());
        self.ast.definitions.push(DefinitionRecord::Interface(id));

        definition_id
    }

    pub fn union_definition(&mut self, definition: UnionDefinitionRecord) -> DefinitionId {
        let id = UnionDefinitionId::new(self.ast.union_definitions.len());
        self.ast.union_definitions.push(definition);

        let definition_id = DefinitionId::new(self.ast.definitions.len());
        self.ast.definitions.push(DefinitionRecord::Union(id));

        definition_id
    }

    pub fn enum_definition(&mut self, definition: EnumDefinitionRecord) -> DefinitionId {
        let id = EnumDefinitionId::new(self.ast.enum_definitions.len());
        self.ast.enum_definitions.push(definition);

        let definition_id = DefinitionId::new(self.ast.definitions.len());
        self.ast.definitions.push(DefinitionRecord::Enum(id));

        definition_id
    }

    pub fn enum_value_definition(
        &mut self,
        definition: EnumValueDefinitionRecord,
    ) -> EnumValueDefinitionId {
        let id = EnumValueDefinitionId::new(self.ast.enum_value_definitions.len());
        self.ast.enum_value_definitions.push(definition);

        id
    }

    pub fn enum_value_definition_range(
        &mut self,
        expected_count: Option<usize>,
    ) -> IdRange<EnumValueDefinitionId> {
        let start = self.enum_value_id_cursor;
        let end = EnumValueDefinitionId::new(self.ast.enum_value_definitions.len());
        self.enum_value_id_cursor = end;
        let range = IdRange::new(start, end);

        assert_eq!(range.len(), expected_count.unwrap_or_default());

        range
    }

    pub fn input_object_definition(
        &mut self,
        definition: InputObjectDefinitionRecord,
    ) -> DefinitionId {
        let id = InputObjectDefinitionId::new(self.ast.input_object_definitions.len());
        self.ast.input_object_definitions.push(definition);

        let definition_id = DefinitionId::new(self.ast.definitions.len());
        self.ast.definitions.push(DefinitionRecord::InputObject(id));

        definition_id
    }

    pub fn input_value_definition(
        &mut self,
        definition: InputValueDefinitionRecord,
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

        assert_eq!(range.len(), expected_count.unwrap_or_default());

        range
    }

    pub fn schema_extension(&mut self, definition: SchemaDefinitionRecord) -> DefinitionId {
        let id = SchemaDefinitionId::new(self.ast.schema_definitions.len());
        self.ast.schema_definitions.push(definition);

        let definition_id = DefinitionId::new(self.ast.definitions.len());
        self.ast
            .definitions
            .push(DefinitionRecord::SchemaExtension(id));

        definition_id
    }

    pub fn scalar_extension(&mut self, definition: ScalarDefinitionRecord) -> DefinitionId {
        let id = ScalarDefinitionId::new(self.ast.scalar_definitions.len());
        self.ast.scalar_definitions.push(definition);

        let definition_id = DefinitionId::new(self.ast.definitions.len());
        self.ast
            .definitions
            .push(DefinitionRecord::ScalarExtension(id));

        definition_id
    }

    pub fn object_extension(&mut self, definition: ObjectDefinitionRecord) -> DefinitionId {
        let id = ObjectDefinitionId::new(self.ast.object_definitions.len());
        self.ast.object_definitions.push(definition);

        let definition_id = DefinitionId::new(self.ast.definitions.len());
        self.ast
            .definitions
            .push(DefinitionRecord::ObjectExtension(id));

        definition_id
    }

    pub fn interface_extension(&mut self, definition: InterfaceDefinitionRecord) -> DefinitionId {
        let id = InterfaceDefinitionId::new(self.ast.interface_definitions.len());
        self.ast.interface_definitions.push(definition);

        let definition_id = DefinitionId::new(self.ast.definitions.len());
        self.ast
            .definitions
            .push(DefinitionRecord::InterfaceExtension(id));

        definition_id
    }

    pub fn union_member(&mut self, member: UnionMemberRecord) -> UnionMemberId {
        let id = UnionMemberId::new(self.ast.union_members.len());
        self.ast.union_members.push(member);
        id
    }

    pub fn union_member_range(&mut self, expected_count: Option<usize>) -> IdRange<UnionMemberId> {
        let start = self.union_member_id_cursor;
        let end = UnionMemberId::new(self.ast.union_members.len());
        self.union_member_id_cursor = end;
        let range = IdRange::new(start, end);

        assert_eq!(range.len(), expected_count.unwrap_or_default());

        range
    }

    pub fn union_extension(&mut self, definition: UnionDefinitionRecord) -> DefinitionId {
        let id = UnionDefinitionId::new(self.ast.union_definitions.len());
        self.ast.union_definitions.push(definition);

        let definition_id = DefinitionId::new(self.ast.definitions.len());
        self.ast
            .definitions
            .push(DefinitionRecord::UnionExtension(id));

        definition_id
    }

    pub fn enum_extension(&mut self, definition: EnumDefinitionRecord) -> DefinitionId {
        let id = EnumDefinitionId::new(self.ast.enum_definitions.len());
        self.ast.enum_definitions.push(definition);

        let definition_id = DefinitionId::new(self.ast.definitions.len());
        self.ast
            .definitions
            .push(DefinitionRecord::EnumExtension(id));

        definition_id
    }

    pub fn input_object_extension(
        &mut self,
        definition: InputObjectDefinitionRecord,
    ) -> DefinitionId {
        let id = InputObjectDefinitionId::new(self.ast.input_object_definitions.len());
        self.ast.input_object_definitions.push(definition);

        let definition_id = DefinitionId::new(self.ast.definitions.len());
        self.ast
            .definitions
            .push(DefinitionRecord::InputObjectExtension(id));

        definition_id
    }

    pub fn directive_definition(&mut self, definition: DirectiveDefinitionRecord) -> DefinitionId {
        let id = DirectiveDefinitionId::new(self.ast.directive_definitions.len());
        self.ast.directive_definitions.push(definition);

        let definition_id = DefinitionId::new(self.ast.definitions.len());
        self.ast.definitions.push(DefinitionRecord::Directive(id));

        definition_id
    }

    pub fn directive_range(&mut self, expected_count: Option<usize>) -> IdRange<DirectiveId> {
        let start = self.directive_id_cursor;
        let end = DirectiveId::new(self.ast.directives.len());
        self.directive_id_cursor = end;
        let range = IdRange::new(start, end);

        assert_eq!(range.len(), expected_count.unwrap_or_default());

        range
    }

    pub fn type_reference(&mut self, ty: TypeRecord) -> TypeId {
        let ty_id = TypeId::new(self.ast.type_references.len());
        self.ast.type_references.push(ty);
        ty_id
    }

    pub fn directive(&mut self, directive: DirectiveRecord) -> DirectiveId {
        let id = DirectiveId::new(self.ast.directives.len());
        self.ast.directives.push(directive);
        id
    }

    pub fn argument(&mut self, argument: ArgumentRecord) -> ArgumentId {
        let id = ArgumentId::new(self.ast.arguments.len());
        self.ast.arguments.push(argument);
        id
    }

    pub fn argument_range(&mut self, expected_count: Option<usize>) -> IdRange<ArgumentId> {
        let start = self.argument_id_cursor;
        let end = ArgumentId::new(self.ast.arguments.len());
        self.argument_id_cursor = end;
        let range = IdRange::new(start, end);

        assert_eq!(range.len(), expected_count.unwrap_or_default());

        range
    }

    pub fn value(&mut self, value: ValueRecord) -> ValueId {
        let id = ValueId::new(self.ast.values.len());
        self.ast.values.push(value);
        id
    }

    pub fn block_string(&mut self, string: &str) -> BlockStringLiteralId {
        let literal_id = BlockStringLiteralId::new(self.ast.block_strings.len());
        self.ast.block_strings.push(string.into());

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

    // TOOD: should this be pub? not sure...
    pub fn intern_owned_string(&mut self, string: String) -> StringId {
        let (id, _) = self.ast.strings.insert_full(string.into());
        StringId::new(id)
    }
}

impl Default for TypeSystemAstWriter {
    fn default() -> Self {
        Self::new()
    }
}
