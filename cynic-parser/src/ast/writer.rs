use crate::ids::StringId;

use super::{ids::*, storage::*, AstLookup};
use super::{Ast, AstDefinition};

#[derive(Default)]
pub struct AstWriter {
    ast: Ast,
}

// TODO: Don't forget the spans etc.
impl AstWriter {
    pub fn new() -> Self {
        AstWriter {
            ast: Default::default(),
        }
    }

    pub fn update(ast: Ast) -> Self {
        AstWriter { ast }
    }

    pub fn finish(self) -> Ast {
        self.ast
    }

    pub fn store_description(&mut self, definition: DefinitionId, description: Option<StringId>) {
        if let Some(description) = description {
            match *self.ast.lookup(definition) {
                AstDefinition::Schema(id) => {
                    self.ast.schema_definitions[id.0].description = Some(description);
                }
                AstDefinition::Scalar(id) => {
                    self.ast.scalar_definitions[id.0].description = Some(description);
                }
                AstDefinition::Object(id) => {
                    self.ast.object_definitions[id.0].description = Some(description);
                }
                AstDefinition::Interface(id) => {
                    self.ast.interface_definitions[id.0].description = Some(description);
                }
                AstDefinition::Union(id) => {
                    self.ast.union_definitions[id.0].description = Some(description);
                }
                AstDefinition::Enum(id) => {
                    self.ast.enum_definitions[id.0].description = Some(description);
                }
                AstDefinition::InputObject(id) => {
                    self.ast.input_object_definitions[id.0].description = Some(description);
                }
                AstDefinition::SchemaExtension(id) => {
                    self.ast.schema_definitions[id.0].description = Some(description);
                }
                AstDefinition::ScalarExtension(id) => {
                    self.ast.scalar_definitions[id.0].description = Some(description);
                }
                AstDefinition::ObjectExtension(id) => {
                    self.ast.object_definitions[id.0].description = Some(description);
                }
                AstDefinition::InterfaceExtension(id) => {
                    self.ast.interface_definitions[id.0].description = Some(description);
                }
                AstDefinition::UnionExtension(id) => {
                    self.ast.union_definitions[id.0].description = Some(description);
                }
                AstDefinition::EnumExtension(id) => {
                    self.ast.enum_definitions[id.0].description = Some(description);
                }
                AstDefinition::InputObjectExtension(id) => {
                    self.ast.input_object_definitions[id.0].description = Some(description);
                }
                AstDefinition::Directive(id) => {
                    self.ast.directive_definitions[id.0].description = Some(description);
                }
            }
        }
    }

    pub fn schema_definition(&mut self, definition: SchemaDefinition) -> DefinitionId {
        let id = SchemaDefinitionId(self.ast.schema_definitions.len());
        self.ast.schema_definitions.push(definition);

        let definition_id = DefinitionId(self.ast.definitions.len());
        self.ast.definitions.push(AstDefinition::Schema(id));

        definition_id
    }

    pub fn scalar_definition(&mut self, definition: ScalarDefinition) -> DefinitionId {
        let id = ScalarDefinitionId(self.ast.scalar_definitions.len());
        self.ast.scalar_definitions.push(definition);

        let definition_id = DefinitionId(self.ast.definitions.len());
        self.ast.definitions.push(AstDefinition::Scalar(id));

        definition_id
    }

    pub fn object_definition(&mut self, definition: ObjectDefinition) -> DefinitionId {
        let id = ObjectDefinitionId(self.ast.object_definitions.len());
        self.ast.object_definitions.push(definition);

        let definition_id = DefinitionId(self.ast.definitions.len());
        self.ast.definitions.push(AstDefinition::Object(id));

        definition_id
    }

    pub fn field_definition(&mut self, definition: FieldDefinition) -> FieldDefinitionId {
        let definition_id = FieldDefinitionId(self.ast.field_definitions.len());
        self.ast.field_definitions.push(definition);

        definition_id
    }

    pub fn interface_definition(&mut self, definition: InterfaceDefinition) -> DefinitionId {
        let id = InterfaceDefinitionId(self.ast.interface_definitions.len());
        self.ast.interface_definitions.push(definition);

        let definition_id = DefinitionId(self.ast.definitions.len());
        self.ast.definitions.push(AstDefinition::Interface(id));

        definition_id
    }

    pub fn union_definition(&mut self, definition: UnionDefinition) -> DefinitionId {
        let id = UnionDefinitionId(self.ast.union_definitions.len());
        self.ast.union_definitions.push(definition);

        let definition_id = DefinitionId(self.ast.definitions.len());
        self.ast.definitions.push(AstDefinition::Union(id));

        definition_id
    }

    pub fn enum_definition(&mut self, definition: EnumDefinition) -> DefinitionId {
        let id = EnumDefinitionId(self.ast.enum_definitions.len());
        self.ast.enum_definitions.push(definition);

        let definition_id = DefinitionId(self.ast.definitions.len());
        self.ast.definitions.push(AstDefinition::Enum(id));

        definition_id
    }

    pub fn enum_value_definition(
        &mut self,
        definition: EnumValueDefinition,
    ) -> EnumValueDefinitionId {
        let id = EnumValueDefinitionId(self.ast.enum_value_definitions.len());
        self.ast.enum_value_definitions.push(definition);

        id
    }

    pub fn input_object_definition(&mut self, definition: InputObjectDefinition) -> DefinitionId {
        let id = InputObjectDefinitionId(self.ast.input_object_definitions.len());
        self.ast.input_object_definitions.push(definition);

        let definition_id = DefinitionId(self.ast.definitions.len());
        self.ast.definitions.push(AstDefinition::InputObject(id));

        definition_id
    }

    pub fn input_value_definition(
        &mut self,
        definition: InputValueDefinition,
    ) -> InputValueDefinitionId {
        let definition_id = InputValueDefinitionId(self.ast.input_value_definitions.len());
        self.ast.input_value_definitions.push(definition);

        definition_id
    }

    pub fn schema_extension(&mut self, definition: SchemaDefinition) -> DefinitionId {
        let id = SchemaDefinitionId(self.ast.schema_definitions.len());
        self.ast.schema_definitions.push(definition);

        let definition_id = DefinitionId(self.ast.definitions.len());
        self.ast
            .definitions
            .push(AstDefinition::SchemaExtension(id));

        definition_id
    }

    pub fn scalar_extension(&mut self, definition: ScalarDefinition) -> DefinitionId {
        let id = ScalarDefinitionId(self.ast.scalar_definitions.len());
        self.ast.scalar_definitions.push(definition);

        let definition_id = DefinitionId(self.ast.definitions.len());
        self.ast
            .definitions
            .push(AstDefinition::ScalarExtension(id));

        definition_id
    }

    pub fn object_extension(&mut self, definition: ObjectDefinition) -> DefinitionId {
        let id = ObjectDefinitionId(self.ast.object_definitions.len());
        self.ast.object_definitions.push(definition);

        let definition_id = DefinitionId(self.ast.definitions.len());
        self.ast
            .definitions
            .push(AstDefinition::ObjectExtension(id));

        definition_id
    }

    pub fn interface_extension(&mut self, definition: InterfaceDefinition) -> DefinitionId {
        let id = InterfaceDefinitionId(self.ast.interface_definitions.len());
        self.ast.interface_definitions.push(definition);

        let definition_id = DefinitionId(self.ast.definitions.len());
        self.ast
            .definitions
            .push(AstDefinition::InterfaceExtension(id));

        definition_id
    }

    pub fn union_extension(&mut self, definition: UnionDefinition) -> DefinitionId {
        let id = UnionDefinitionId(self.ast.union_definitions.len());
        self.ast.union_definitions.push(definition);

        let definition_id = DefinitionId(self.ast.definitions.len());
        self.ast.definitions.push(AstDefinition::UnionExtension(id));

        definition_id
    }

    pub fn enum_extension(&mut self, definition: EnumDefinition) -> DefinitionId {
        let id = EnumDefinitionId(self.ast.enum_definitions.len());
        self.ast.enum_definitions.push(definition);

        let definition_id = DefinitionId(self.ast.definitions.len());
        self.ast.definitions.push(AstDefinition::EnumExtension(id));

        definition_id
    }

    pub fn input_object_extension(&mut self, definition: InputObjectDefinition) -> DefinitionId {
        let id = InputObjectDefinitionId(self.ast.input_object_definitions.len());
        self.ast.input_object_definitions.push(definition);

        let definition_id = DefinitionId(self.ast.definitions.len());
        self.ast
            .definitions
            .push(AstDefinition::InputObjectExtension(id));

        definition_id
    }

    pub fn directive_definition(&mut self, definition: DirectiveDefinition) -> DefinitionId {
        let id = DirectiveDefinitionId(self.ast.directive_definitions.len());
        self.ast.directive_definitions.push(definition);

        let definition_id = DefinitionId(self.ast.definitions.len());
        self.ast.definitions.push(AstDefinition::Directive(id));

        definition_id
    }

    pub fn type_reference(&mut self, ty: Type) -> TypeId {
        let ty_id = TypeId(self.ast.type_references.len());
        self.ast.type_references.push(ty);
        ty_id
    }

    pub fn directive(&mut self, directive: Directive) -> DirectiveId {
        let id = DirectiveId(self.ast.directives.len());
        self.ast.directives.push(directive);
        id
    }

    pub fn argument(&mut self, argument: Argument) -> ArgumentId {
        let id = ArgumentId(self.ast.arguments.len());
        self.ast.arguments.push(argument);
        id
    }

    pub fn value(&mut self, value: Value) -> ValueId {
        let id = ValueId(self.ast.values.len());
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
        StringId(id)
    }
}
