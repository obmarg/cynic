use std::sync::Arc;

use indexmap::IndexSet;

use crate::common::IdRange;

use super::{ExecutableDocument, ids::*, storage::*};

pub struct ExecutableAstWriter {
    pub values: crate::values::writer::ValueWriter,

    strings: IndexSet<Box<str>>,
    block_strings: Vec<Box<str>>,

    definitions: Vec<ExecutableDefinitionRecord>,
    operations: Vec<OperationDefinitionRecord>,
    fragments: Vec<FragmentDefinitionRecord>,

    selections: Vec<SelectionRecord>,
    field_selections: Vec<FieldSelectionRecord>,
    inline_fragments: Vec<InlineFragmentRecord>,
    fragment_spreads: Vec<FragmentSpreadRecord>,

    directives: Vec<DirectiveRecord>,
    arguments: Vec<ArgumentRecord>,
    variables: Vec<VariableDefinitionRecord>,

    types: Vec<TypeRecord>,

    directive_cursor: DirectiveId,
    variable_definition_cursor: VariableDefinitionId,
}

impl Default for ExecutableAstWriter {
    fn default() -> Self {
        Self {
            strings: Default::default(),
            block_strings: Default::default(),
            definitions: Default::default(),
            operations: Default::default(),
            fragments: Default::default(),
            selections: Default::default(),
            field_selections: Default::default(),
            inline_fragments: Default::default(),
            fragment_spreads: Default::default(),
            directives: Default::default(),
            arguments: Default::default(),
            variables: Default::default(),
            types: Default::default(),
            values: Default::default(),
            directive_cursor: DirectiveId::new(0),
            variable_definition_cursor: VariableDefinitionId::new(0),
        }
    }
}

impl ExecutableAstWriter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn finish(self) -> ExecutableDocument {
        // TODO: Possibly assert things in here for safety...
        let ExecutableAstWriter {
            strings,
            block_strings,
            definitions,
            operations,
            fragments,
            selections,
            field_selections,
            inline_fragments,
            fragment_spreads,
            directives,
            arguments,
            variables,
            types,
            values,
            directive_cursor: _,
            variable_definition_cursor: _,
        } = self;

        let strings = Arc::new(strings);
        let values = values.finish(Arc::clone(&strings));

        ExecutableDocument {
            strings,
            block_strings,
            definitions,
            operations,
            fragments,
            selections,
            field_selections,
            inline_fragments,
            fragment_spreads,
            directives,
            arguments,
            variables,
            types,
            values,
        }
    }

    pub fn operation_definition(
        &mut self,
        definition: OperationDefinitionRecord,
    ) -> ExecutableDefinitionId {
        let id = OperationDefinitionId::new(self.operations.len());
        self.operations.push(definition);

        let definition_id = ExecutableDefinitionId::new(self.definitions.len());
        self.definitions
            .push(ExecutableDefinitionRecord::Operation(id));

        definition_id
    }

    pub fn fragment_definition(
        &mut self,
        definition: FragmentDefinitionRecord,
    ) -> ExecutableDefinitionId {
        let id = FragmentDefinitionId::new(self.fragments.len());
        self.fragments.push(definition);

        let definition_id = ExecutableDefinitionId::new(self.definitions.len());
        self.definitions
            .push(ExecutableDefinitionRecord::Fragment(id));

        definition_id
    }

    pub fn variable_definition(
        &mut self,
        record: VariableDefinitionRecord,
    ) -> VariableDefinitionId {
        let id = VariableDefinitionId::new(self.variables.len());
        self.variables.push(record);
        id
    }

    pub fn variable_definition_range(
        &mut self,
        expected_count: Option<usize>,
    ) -> IdRange<VariableDefinitionId> {
        let start = self.variable_definition_cursor;
        let end = VariableDefinitionId::new(self.variables.len());
        self.variable_definition_cursor = end;
        let range = IdRange::new(start, end);

        assert_eq!(range.len(), expected_count.unwrap_or_default());

        range
    }

    pub fn type_reference(&mut self, ty: TypeRecord) -> TypeId {
        let ty_id = TypeId::new(self.types.len());
        self.types.push(ty);
        ty_id
    }

    pub fn selection_set(
        &mut self,
        mut selection_set: Vec<SelectionRecord>,
    ) -> IdRange<SelectionId> {
        let start_range = SelectionId::new(self.selections.len());
        self.selections.append(&mut selection_set);
        let end_range = SelectionId::new(self.selections.len());

        IdRange::new(start_range, end_range)
    }

    pub fn field_selection(&mut self, record: FieldSelectionRecord) -> FieldSelectionId {
        let id = FieldSelectionId::new(self.field_selections.len());
        self.field_selections.push(record);
        id
    }

    pub fn fragment_spread(&mut self, record: FragmentSpreadRecord) -> FragmentSpreadId {
        let id = FragmentSpreadId::new(self.fragment_spreads.len());
        self.fragment_spreads.push(record);
        id
    }

    pub fn inline_fragment(&mut self, record: InlineFragmentRecord) -> InlineFragmentId {
        let id = InlineFragmentId::new(self.inline_fragments.len());
        self.inline_fragments.push(record);
        id
    }

    pub fn arguments(&mut self, mut records: Vec<ArgumentRecord>) -> IdRange<ArgumentId> {
        let start = ArgumentId::new(self.arguments.len());
        self.arguments.append(&mut records);
        let end = ArgumentId::new(self.arguments.len());

        IdRange::new(start, end)
    }

    pub fn directive(&mut self, directive: DirectiveRecord) -> DirectiveId {
        let id = DirectiveId::new(self.directives.len());
        self.directives.push(directive);
        id
    }

    pub fn directive_range(&mut self, expected_count: Option<usize>) -> IdRange<DirectiveId> {
        let start = self.directive_cursor;
        let end = DirectiveId::new(self.directives.len());
        self.directive_cursor = end;
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
