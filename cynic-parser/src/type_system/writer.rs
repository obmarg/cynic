use std::sync::Arc;

use indexmap::IndexSet;

use crate::common::IdRange;
use crate::values::writer::ValueWriter;

use super::{ids::*, storage::*};
use super::{DefinitionRecord, TypeSystemDocument};

pub struct TypeSystemAstWriter {
    pub(crate) values: crate::values::writer::ValueWriter,

    strings: IndexSet<Box<str>>,
    block_strings: Vec<Box<str>>,

    definitions: Vec<DefinitionRecord>,

    schema_definitions: Vec<SchemaDefinitionRecord>,
    scalar_definitions: Vec<ScalarDefinitionRecord>,
    object_definitions: Vec<ObjectDefinitionRecord>,
    interface_definitions: Vec<InterfaceDefinitionRecord>,
    union_definitions: Vec<UnionDefinitionRecord>,
    enum_definitions: Vec<EnumDefinitionRecord>,
    input_object_definitions: Vec<InputObjectDefinitionRecord>,
    directive_definitions: Vec<DirectiveDefinitionRecord>,

    root_operation_definitions: Vec<RootOperationTypeDefinitionRecord>,

    field_definitions: Vec<FieldDefinitionRecord>,
    input_value_definitions: Vec<InputValueDefinitionRecord>,
    enum_value_definitions: Vec<EnumValueDefinitionRecord>,
    union_members: Vec<UnionMemberRecord>,

    type_references: Vec<TypeRecord>,

    directives: Vec<DirectiveRecord>,
    arguments: Vec<ArgumentRecord>,
    descriptions: Vec<DescriptionRecord>,

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
            strings: Default::default(),
            block_strings: Default::default(),
            definitions: Default::default(),
            schema_definitions: Default::default(),
            scalar_definitions: Default::default(),
            object_definitions: Default::default(),
            interface_definitions: Default::default(),
            union_definitions: Default::default(),
            enum_definitions: Default::default(),
            input_object_definitions: Default::default(),
            directive_definitions: Default::default(),
            root_operation_definitions: Default::default(),
            field_definitions: Default::default(),
            input_value_definitions: Default::default(),
            enum_value_definitions: Default::default(),
            union_members: Default::default(),
            type_references: Default::default(),
            directives: Default::default(),
            arguments: Default::default(),
            descriptions: Default::default(),

            values: Default::default(),

            field_id_cursor: FieldDefinitionId::new(0),
            input_value_id_cursor: InputValueDefinitionId::new(0),
            enum_value_id_cursor: EnumValueDefinitionId::new(0),
            directive_id_cursor: DirectiveId::new(0),
            argument_id_cursor: ArgumentId::new(0),
            union_member_id_cursor: UnionMemberId::new(0),
        }
    }

    pub fn update(ast: TypeSystemDocument) -> Self {
        let TypeSystemDocument {
            strings,
            block_strings,
            definitions,
            schema_definitions,
            scalar_definitions,
            object_definitions,
            interface_definitions,
            union_definitions,
            enum_definitions,
            input_object_definitions,
            directive_definitions,
            root_operation_definitions,
            field_definitions,
            input_value_definitions,
            enum_value_definitions,
            union_members,
            type_references,
            directives,
            arguments,
            descriptions,
            values,
        } = ast;

        let values = ValueWriter::update(values);
        let strings = Arc::unwrap_or_clone(strings);

        TypeSystemAstWriter {
            field_id_cursor: FieldDefinitionId::new(field_definitions.len()),
            input_value_id_cursor: InputValueDefinitionId::new(input_value_definitions.len()),
            directive_id_cursor: DirectiveId::new(directives.len()),
            enum_value_id_cursor: EnumValueDefinitionId::new(enum_value_definitions.len()),
            argument_id_cursor: ArgumentId::new(arguments.len()),
            union_member_id_cursor: UnionMemberId::new(union_members.len()),

            strings,
            block_strings,
            definitions,
            schema_definitions,
            scalar_definitions,
            object_definitions,
            interface_definitions,
            union_definitions,
            enum_definitions,
            input_object_definitions,
            directive_definitions,
            root_operation_definitions,
            field_definitions,
            input_value_definitions,
            enum_value_definitions,
            union_members,
            type_references,
            directives,
            arguments,
            descriptions,

            values,
        }
    }

    pub fn finish(self) -> TypeSystemDocument {
        // TODO: Possibly assert things in here for safety...
        let Self {
            strings,
            block_strings,
            definitions,
            schema_definitions,
            scalar_definitions,
            object_definitions,
            interface_definitions,
            union_definitions,
            enum_definitions,
            input_object_definitions,
            directive_definitions,
            root_operation_definitions,
            field_definitions,
            input_value_definitions,
            enum_value_definitions,
            union_members,
            type_references,
            directives,
            arguments,
            descriptions,

            values,

            field_id_cursor: _,
            input_value_id_cursor: _,
            enum_value_id_cursor: _,
            directive_id_cursor: _,
            argument_id_cursor: _,
            union_member_id_cursor: _,
        } = self;

        let strings = Arc::new(strings);
        let values = values.finish(Arc::clone(&strings));

        TypeSystemDocument {
            strings,
            block_strings,
            definitions,
            schema_definitions,
            scalar_definitions,
            object_definitions,
            interface_definitions,
            union_definitions,
            enum_definitions,
            input_object_definitions,
            directive_definitions,
            root_operation_definitions,
            field_definitions,
            input_value_definitions,
            enum_value_definitions,
            union_members,
            type_references,
            directives,
            arguments,
            descriptions,
            values,
        }
    }

    pub fn store_description(
        &mut self,
        definition: DefinitionId,
        description: Option<DescriptionId>,
    ) {
        if let Some(description) = description {
            match self.definitions[definition.get()] {
                DefinitionRecord::Schema(id) | DefinitionRecord::SchemaExtension(id) => {
                    self.schema_definitions[id.get()].description = Some(description);
                }
                DefinitionRecord::Scalar(id) | DefinitionRecord::ScalarExtension(id) => {
                    self.scalar_definitions[id.get()].description = Some(description);
                }
                DefinitionRecord::Object(id) | DefinitionRecord::ObjectExtension(id) => {
                    self.object_definitions[id.get()].description = Some(description);
                }
                DefinitionRecord::Interface(id) | DefinitionRecord::InterfaceExtension(id) => {
                    self.interface_definitions[id.get()].description = Some(description);
                }
                DefinitionRecord::Union(id) | DefinitionRecord::UnionExtension(id) => {
                    self.union_definitions[id.get()].description = Some(description);
                }
                DefinitionRecord::Enum(id) | DefinitionRecord::EnumExtension(id) => {
                    self.enum_definitions[id.get()].description = Some(description);
                }
                DefinitionRecord::InputObject(id) | DefinitionRecord::InputObjectExtension(id) => {
                    self.input_object_definitions[id.get()].description = Some(description);
                }
                DefinitionRecord::Directive(id) => {
                    self.directive_definitions[id.get()].description = Some(description);
                }
            }
        }
    }

    pub fn description(&mut self, description: DescriptionRecord) -> DescriptionId {
        let id = DescriptionId::new(self.descriptions.len());
        self.descriptions.push(description);
        id
    }

    pub fn schema_definition(&mut self, definition: SchemaDefinitionRecord) -> DefinitionId {
        let id = SchemaDefinitionId::new(self.schema_definitions.len());
        self.schema_definitions.push(definition);

        let definition_id = DefinitionId::new(self.definitions.len());
        self.definitions.push(DefinitionRecord::Schema(id));

        definition_id
    }

    pub fn root_operation_definitions(
        &mut self,
        definitions: Vec<RootOperationTypeDefinitionRecord>,
    ) -> IdRange<RootOperationTypeDefinitionId> {
        let start_id = RootOperationTypeDefinitionId::new(self.root_operation_definitions.len());

        self.root_operation_definitions.extend(definitions);
        let end_id = RootOperationTypeDefinitionId::new(self.root_operation_definitions.len());

        IdRange::new(start_id, end_id)
    }

    pub fn scalar_definition(&mut self, definition: ScalarDefinitionRecord) -> DefinitionId {
        let id = ScalarDefinitionId::new(self.scalar_definitions.len());
        self.scalar_definitions.push(definition);

        let definition_id = DefinitionId::new(self.definitions.len());
        self.definitions.push(DefinitionRecord::Scalar(id));

        definition_id
    }

    pub fn object_definition(&mut self, definition: ObjectDefinitionRecord) -> DefinitionId {
        // TODO: Maybe assert in here too?

        let id = ObjectDefinitionId::new(self.object_definitions.len());
        self.object_definitions.push(definition);

        let definition_id = DefinitionId::new(self.definitions.len());
        self.definitions.push(DefinitionRecord::Object(id));

        definition_id
    }

    pub fn field_definition(&mut self, definition: FieldDefinitionRecord) -> FieldDefinitionId {
        let definition_id = FieldDefinitionId::new(self.field_definitions.len());
        self.field_definitions.push(definition);

        definition_id
    }

    pub fn field_definition_range(
        &mut self,
        expected_count: Option<usize>,
    ) -> IdRange<FieldDefinitionId> {
        let start = self.field_id_cursor;
        let end = FieldDefinitionId::new(self.field_definitions.len());
        self.field_id_cursor = end;
        let range = IdRange::new(start, end);

        if let Some(expected_count) = expected_count {
            assert_eq!(range.len(), expected_count);
        }

        range
    }

    pub fn interface_definition(&mut self, definition: InterfaceDefinitionRecord) -> DefinitionId {
        let id = InterfaceDefinitionId::new(self.interface_definitions.len());
        self.interface_definitions.push(definition);

        let definition_id = DefinitionId::new(self.definitions.len());
        self.definitions.push(DefinitionRecord::Interface(id));

        definition_id
    }

    pub fn union_definition(&mut self, definition: UnionDefinitionRecord) -> DefinitionId {
        let id = UnionDefinitionId::new(self.union_definitions.len());
        self.union_definitions.push(definition);

        let definition_id = DefinitionId::new(self.definitions.len());
        self.definitions.push(DefinitionRecord::Union(id));

        definition_id
    }

    pub fn enum_definition(&mut self, definition: EnumDefinitionRecord) -> DefinitionId {
        let id = EnumDefinitionId::new(self.enum_definitions.len());
        self.enum_definitions.push(definition);

        let definition_id = DefinitionId::new(self.definitions.len());
        self.definitions.push(DefinitionRecord::Enum(id));

        definition_id
    }

    pub fn enum_value_definition(
        &mut self,
        definition: EnumValueDefinitionRecord,
    ) -> EnumValueDefinitionId {
        let id = EnumValueDefinitionId::new(self.enum_value_definitions.len());
        self.enum_value_definitions.push(definition);

        id
    }

    pub fn enum_value_definition_range(
        &mut self,
        expected_count: Option<usize>,
    ) -> IdRange<EnumValueDefinitionId> {
        let start = self.enum_value_id_cursor;
        let end = EnumValueDefinitionId::new(self.enum_value_definitions.len());
        self.enum_value_id_cursor = end;
        let range = IdRange::new(start, end);

        assert_eq!(range.len(), expected_count.unwrap_or_default());

        range
    }

    pub fn input_object_definition(
        &mut self,
        definition: InputObjectDefinitionRecord,
    ) -> DefinitionId {
        let id = InputObjectDefinitionId::new(self.input_object_definitions.len());
        self.input_object_definitions.push(definition);

        let definition_id = DefinitionId::new(self.definitions.len());
        self.definitions.push(DefinitionRecord::InputObject(id));

        definition_id
    }

    pub fn input_value_definition(
        &mut self,
        definition: InputValueDefinitionRecord,
    ) -> InputValueDefinitionId {
        let definition_id = InputValueDefinitionId::new(self.input_value_definitions.len());
        self.input_value_definitions.push(definition);

        definition_id
    }

    pub fn input_value_definition_range(
        &mut self,
        expected_count: Option<usize>,
    ) -> IdRange<InputValueDefinitionId> {
        let start = self.input_value_id_cursor;
        let end = InputValueDefinitionId::new(self.input_value_definitions.len());
        self.input_value_id_cursor = end;
        let range = IdRange::new(start, end);

        assert_eq!(range.len(), expected_count.unwrap_or_default());

        range
    }

    pub fn schema_extension(&mut self, definition: SchemaDefinitionRecord) -> DefinitionId {
        let id = SchemaDefinitionId::new(self.schema_definitions.len());
        self.schema_definitions.push(definition);

        let definition_id = DefinitionId::new(self.definitions.len());
        self.definitions.push(DefinitionRecord::SchemaExtension(id));

        definition_id
    }

    pub fn scalar_extension(&mut self, definition: ScalarDefinitionRecord) -> DefinitionId {
        let id = ScalarDefinitionId::new(self.scalar_definitions.len());
        self.scalar_definitions.push(definition);

        let definition_id = DefinitionId::new(self.definitions.len());
        self.definitions.push(DefinitionRecord::ScalarExtension(id));

        definition_id
    }

    pub fn object_extension(&mut self, definition: ObjectDefinitionRecord) -> DefinitionId {
        let id = ObjectDefinitionId::new(self.object_definitions.len());
        self.object_definitions.push(definition);

        let definition_id = DefinitionId::new(self.definitions.len());
        self.definitions.push(DefinitionRecord::ObjectExtension(id));

        definition_id
    }

    pub fn interface_extension(&mut self, definition: InterfaceDefinitionRecord) -> DefinitionId {
        let id = InterfaceDefinitionId::new(self.interface_definitions.len());
        self.interface_definitions.push(definition);

        let definition_id = DefinitionId::new(self.definitions.len());
        self.definitions
            .push(DefinitionRecord::InterfaceExtension(id));

        definition_id
    }

    pub fn union_member(&mut self, member: UnionMemberRecord) -> UnionMemberId {
        let id = UnionMemberId::new(self.union_members.len());
        self.union_members.push(member);
        id
    }

    pub fn union_member_range(&mut self, expected_count: Option<usize>) -> IdRange<UnionMemberId> {
        let start = self.union_member_id_cursor;
        let end = UnionMemberId::new(self.union_members.len());
        self.union_member_id_cursor = end;
        let range = IdRange::new(start, end);

        assert_eq!(range.len(), expected_count.unwrap_or_default());

        range
    }

    pub fn union_extension(&mut self, definition: UnionDefinitionRecord) -> DefinitionId {
        let id = UnionDefinitionId::new(self.union_definitions.len());
        self.union_definitions.push(definition);

        let definition_id = DefinitionId::new(self.definitions.len());
        self.definitions.push(DefinitionRecord::UnionExtension(id));

        definition_id
    }

    pub fn enum_extension(&mut self, definition: EnumDefinitionRecord) -> DefinitionId {
        let id = EnumDefinitionId::new(self.enum_definitions.len());
        self.enum_definitions.push(definition);

        let definition_id = DefinitionId::new(self.definitions.len());
        self.definitions.push(DefinitionRecord::EnumExtension(id));

        definition_id
    }

    pub fn input_object_extension(
        &mut self,
        definition: InputObjectDefinitionRecord,
    ) -> DefinitionId {
        let id = InputObjectDefinitionId::new(self.input_object_definitions.len());
        self.input_object_definitions.push(definition);

        let definition_id = DefinitionId::new(self.definitions.len());
        self.definitions
            .push(DefinitionRecord::InputObjectExtension(id));

        definition_id
    }

    pub fn directive_definition(&mut self, definition: DirectiveDefinitionRecord) -> DefinitionId {
        let id = DirectiveDefinitionId::new(self.directive_definitions.len());
        self.directive_definitions.push(definition);

        let definition_id = DefinitionId::new(self.definitions.len());
        self.definitions.push(DefinitionRecord::Directive(id));

        definition_id
    }

    pub fn directive_range(&mut self, expected_count: Option<usize>) -> IdRange<DirectiveId> {
        let start = self.directive_id_cursor;
        let end = DirectiveId::new(self.directives.len());
        self.directive_id_cursor = end;
        let range = IdRange::new(start, end);

        assert_eq!(range.len(), expected_count.unwrap_or_default());

        range
    }

    pub fn type_reference(&mut self, ty: TypeRecord) -> TypeId {
        let ty_id = TypeId::new(self.type_references.len());
        self.type_references.push(ty);
        ty_id
    }

    pub fn directive(&mut self, directive: DirectiveRecord) -> DirectiveId {
        let id = DirectiveId::new(self.directives.len());
        self.directives.push(directive);
        id
    }

    pub fn argument(&mut self, argument: ArgumentRecord) -> ArgumentId {
        let id = ArgumentId::new(self.arguments.len());
        self.arguments.push(argument);
        id
    }

    pub fn argument_range(&mut self, expected_count: Option<usize>) -> IdRange<ArgumentId> {
        let start = self.argument_id_cursor;
        let end = ArgumentId::new(self.arguments.len());
        self.argument_id_cursor = end;
        let range = IdRange::new(start, end);

        assert_eq!(range.len(), expected_count.unwrap_or_default());

        range
    }

    pub fn block_string(&mut self, string: &str) -> BlockStringLiteralId {
        let literal_id = BlockStringLiteralId::new(self.block_strings.len());
        self.block_strings.push(string.into());

        literal_id
    }

    pub fn ident(&mut self, ident: &str) -> StringId {
        self.intern_string(ident)
    }

    // TOOD: should this be pub? not sure...
    pub fn intern_string(&mut self, string: &str) -> StringId {
        let (id, _) = self.strings.insert_full(string.into());
        StringId::new(id)
    }

    // TOOD: should this be pub? not sure...
    pub fn intern_owned_string(&mut self, string: String) -> StringId {
        let (id, _) = self.strings.insert_full(string.into());
        StringId::new(id)
    }
}

impl Default for TypeSystemAstWriter {
    fn default() -> Self {
        Self::new()
    }
}
