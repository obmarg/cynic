use crate::common::IdRange;

use super::{
    definition::ExecutableDefinitionRecord, directive::DirectiveRecord, ids::*, storage::*,
    variable::VariableDefinitionRecord, ExecutableDocument,
};

pub struct ExecutableAstWriter {
    ast: ExecutableDocument,
    directive_cursor: DirectiveId,
    variable_definition_cursor: VariableDefinitionId,
}

impl ExecutableAstWriter {
    pub fn new() -> Self {
        ExecutableAstWriter {
            ast: ExecutableDocument::default(),
            directive_cursor: DirectiveId::new(0),
            variable_definition_cursor: VariableDefinitionId::new(0),
        }
    }

    pub fn finish(self) -> ExecutableDocument {
        // TODO: Possibly assert things in here for safety...
        self.ast
    }

    pub fn operation_definition(
        &mut self,
        definition: OperationDefinitionRecord,
    ) -> ExecutableDefinitionId {
        let id = OperationDefinitionId::new(self.ast.operations.len());
        self.ast.operations.push(definition);

        let definition_id = ExecutableDefinitionId::new(self.ast.definitions.len());
        self.ast
            .definitions
            .push(ExecutableDefinitionRecord::Operation(id));

        definition_id
    }

    pub fn fragment_definition(
        &mut self,
        definition: FragmentDefinitionRecord,
    ) -> ExecutableDefinitionId {
        let id = FragmentDefinitionId::new(self.ast.fragments.len());
        self.ast.fragments.push(definition);

        let definition_id = ExecutableDefinitionId::new(self.ast.definitions.len());
        self.ast
            .definitions
            .push(ExecutableDefinitionRecord::Fragment(id));

        definition_id
    }

    pub fn variable_definition(
        &mut self,
        record: VariableDefinitionRecord,
    ) -> VariableDefinitionId {
        let id = VariableDefinitionId::new(self.ast.variables.len());
        self.ast.variables.push(record);
        id
    }

    pub fn variable_definition_range(
        &mut self,
        expected_count: Option<usize>,
    ) -> IdRange<VariableDefinitionId> {
        let start = self.variable_definition_cursor;
        let end = VariableDefinitionId::new(self.ast.variables.len());
        self.variable_definition_cursor = end;
        let range = IdRange::new(start, end);

        assert_eq!(range.len(), expected_count.unwrap_or_default());

        range
    }

    pub fn type_reference(&mut self, ty: TypeRecord) -> TypeId {
        let ty_id = TypeId::new(self.ast.types.len());
        self.ast.types.push(ty);
        ty_id
    }

    pub fn selection_set(
        &mut self,
        mut selection_set: Vec<SelectionRecord>,
    ) -> IdRange<SelectionId> {
        let start_range = SelectionId::new(self.ast.selections.len());
        self.ast.selections.append(&mut selection_set);
        let end_range = SelectionId::new(self.ast.selections.len());

        IdRange::new(start_range, end_range)
    }

    pub fn field_selection(&mut self, record: FieldSelectionRecord) -> FieldSelectionId {
        let id = FieldSelectionId::new(self.ast.field_selections.len());
        self.ast.field_selections.push(record);
        id
    }

    pub fn fragment_spread(&mut self, record: FragmentSpreadRecord) -> FragmentSpreadId {
        let id = FragmentSpreadId::new(self.ast.fragment_spreads.len());
        self.ast.fragment_spreads.push(record);
        id
    }

    pub fn inline_fragment(&mut self, record: InlineFragmentRecord) -> InlineFragmentId {
        let id = InlineFragmentId::new(self.ast.inline_fragments.len());
        self.ast.inline_fragments.push(record);
        id
    }

    pub fn arguments(&mut self, mut records: Vec<ArgumentRecord>) -> IdRange<ArgumentId> {
        let start = ArgumentId::new(self.ast.arguments.len());
        self.ast.arguments.append(&mut records);
        let end = ArgumentId::new(self.ast.arguments.len());

        IdRange::new(start, end)
    }

    pub fn directive(&mut self, directive: DirectiveRecord) -> DirectiveId {
        let id = DirectiveId::new(self.ast.directives.len());
        self.ast.directives.push(directive);
        id
    }

    pub fn directive_range(&mut self, expected_count: Option<usize>) -> IdRange<DirectiveId> {
        let start = self.directive_cursor;
        let end = DirectiveId::new(self.ast.directives.len());
        self.directive_cursor = end;
        let range = IdRange::new(start, end);

        assert_eq!(range.len(), expected_count.unwrap_or_default());

        range
    }

    pub fn value(&mut self, record: ValueRecord) -> ValueId {
        let id = ValueId::new(self.ast.values.len());
        self.ast.values.push(record);
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

impl Default for ExecutableAstWriter {
    fn default() -> Self {
        Self::new()
    }
}
