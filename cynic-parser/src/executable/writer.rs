use crate::{common::IdRange, AstLookup};

use super::{
    definition::ExecutableDefinitionRecord, directive::DirectiveRecord, ids::*, storage::*,
    variable::VariableDefinitionRecord, Ast,
};

pub struct AstWriter {
    ast: Ast,
    argument_cursor: ArgumentId,
    directive_cursor: DirectiveId,
    variable_definition_cursor: VariableDefinitionId,
    // field_id_cursor: FieldDefinitionId,
    // input_value_id_cursor: InputValueDefinitionId,
    // directive_id_cursor: DirectiveId,
}

impl AstWriter {
    pub fn new() -> Self {
        AstWriter {
            ast: Ast::default(),
            argument_cursor: ArgumentId::new(0),
            directive_cursor: DirectiveId::new(0),
            variable_definition_cursor: VariableDefinitionId::new(0),
            // field_id_cursor: FieldDefinitionId::new(0),
            // input_value_id_cursor: InputValueDefinitionId::new(0),
            // directive_id_cursor: DirectiveId::new(0),
        }
    }

    pub fn finish(self) -> Ast {
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

    pub fn selection_set(&mut self, selection_set: Vec<SelectionRecord>) -> IdRange<SelectionId> {
        todo!()
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

    pub fn argument(&mut self, record: ArgumentRecord) -> ArgumentId {
        let id = ArgumentId::new(self.ast.arguments.len());
        self.ast.arguments.push(record);
        id
    }

    pub fn argument_range(&mut self, expected_count: Option<usize>) -> IdRange<ArgumentId> {
        let start = self.argument_cursor;
        let end = ArgumentId::new(self.ast.arguments.len());
        self.argument_cursor = end;
        let range = IdRange::new(start, end);

        assert_eq!(range.len(), expected_count.unwrap_or_default());

        range
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

    pub fn ident(&mut self, ident: &str) -> StringId {
        self.intern_string(ident)
    }

    // TOOD: should this be pub? not sure...
    pub fn intern_string(&mut self, string: &str) -> StringId {
        let (id, _) = self.ast.strings.insert_full(string.into());
        StringId::new(id)
    }
}
