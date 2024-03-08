use crate::AstLookup;

use super::{definition::ExecutableDefinitionRecord, ids::*, storage::*, Ast};

pub struct AstWriter {
    ast: Ast,
    // field_id_cursor: FieldDefinitionId,
    // input_value_id_cursor: InputValueDefinitionId,
    // directive_id_cursor: DirectiveId,
}

impl AstWriter {
    pub fn new() -> Self {
        AstWriter {
            ast: Ast::default(),
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

    pub fn ident(&mut self, ident: &str) -> StringId {
        self.intern_string(ident)
    }

    // TOOD: should this be pub? not sure...
    pub fn intern_string(&mut self, string: &str) -> StringId {
        let (id, _) = self.ast.strings.insert_full(string.into());
        StringId::new(id)
    }
}
