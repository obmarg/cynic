use std::str::FromStr;

use ids::*;
use indexmap::IndexSet;

pub mod ids;
pub mod readers;
mod span;

pub use readers::{Definition, TypeDefinition, ValueReader};
pub use span::Span;

#[derive(Default)]
pub struct Ast {
    strings: IndexSet<Box<str>>,

    definitions: Vec<AstDefinition>,

    schema_definitions: Vec<SchemaDefinition>,
    scalar_definitions: Vec<ScalarDefinition>,
    object_definitions: Vec<ObjectDefinition>,
    interface_definitions: Vec<InterfaceDefinition>,
    union_definitions: Vec<UnionDefinition>,
    enum_definitions: Vec<EnumDefinition>,
    input_object_definitions: Vec<InputObjectDefinition>,
    directive_definitions: Vec<DirectiveDefinition>,

    field_definitions: Vec<FieldDefinition>,
    input_value_definitions: Vec<InputValueDefinition>,
    enum_value_definitions: Vec<EnumValueDefinition>,

    type_references: Vec<Type>,

    string_literals: Vec<StringLiteral>,

    values: Vec<Value>,
    directives: Vec<Directive>,
    arguments: Vec<Argument>,
}

#[derive(Clone, Copy)]
pub enum AstDefinition {
    Schema(SchemaDefinitionId),
    Scalar(ScalarDefinitionId),
    Object(ObjectDefinitionId),
    Interface(InterfaceDefinitionId),
    Union(UnionDefinitionId),
    Enum(EnumDefinitionId),
    InputObject(InputObjectDefinitionId),
    SchemaExtension(SchemaDefinitionId),
    ScalarExtension(ScalarDefinitionId),
    ObjectExtension(ObjectDefinitionId),
    InterfaceExtension(InterfaceDefinitionId),
    UnionExtension(UnionDefinitionId),
    EnumExtension(EnumDefinitionId),
    InputObjectExtension(InputObjectDefinitionId),
    Directive(DirectiveDefinitionId),
}

pub struct SchemaDefinition {
    pub description: Option<StringId>,
    pub roots: Vec<RootOperationTypeDefinition>,
}

pub struct ScalarDefinition {
    pub name: StringId,
    pub description: Option<StringId>,
    pub directives: Vec<DirectiveId>,
    pub span: Span,
}

pub struct ObjectDefinition {
    pub name: StringId,
    pub description: Option<StringId>,
    pub fields: Vec<FieldDefinitionId>,
    pub directives: Vec<DirectiveId>,
    pub implements: Vec<StringId>,
    pub span: Span,
}

pub struct FieldDefinition {
    pub name: StringId,
    pub ty: TypeId,
    pub arguments: Vec<InputValueDefinitionId>,
    pub description: Option<StringId>,
    pub directives: Vec<DirectiveId>,
    pub span: Span,
}

pub struct InterfaceDefinition {
    pub name: StringId,
    pub description: Option<StringId>,
    pub fields: Vec<FieldDefinitionId>,
    pub directives: Vec<DirectiveId>,
    pub implements: Vec<StringId>,
    pub span: Span,
}

pub struct UnionDefinition {
    pub name: StringId,
    pub description: Option<StringId>,
    pub members: Vec<StringId>,
    pub directives: Vec<DirectiveId>,
    pub span: Span,
}

pub struct EnumDefinition {
    pub name: StringId,
    pub description: Option<StringId>,
    pub values: Vec<EnumValueDefinitionId>,
    pub directives: Vec<DirectiveId>,
    pub span: Span,
}

pub struct EnumValueDefinition {
    pub value: StringId,
    pub description: Option<StringId>,
    pub directives: Vec<DirectiveId>,
    pub span: Span,
}

pub struct InputObjectDefinition {
    pub name: StringId,
    pub description: Option<StringId>,
    pub fields: Vec<InputValueDefinitionId>,
    pub directives: Vec<DirectiveId>,
    pub span: Span,
}

pub struct InputValueDefinition {
    pub name: StringId,
    pub ty: TypeId,
    pub description: Option<StringId>,
    pub default: Option<ValueId>,
    pub directives: Vec<DirectiveId>,
    pub span: Span,
}

pub struct DirectiveDefinition {
    pub name: StringId,
    pub description: Option<StringId>,
    pub arguments: Vec<InputValueDefinitionId>,
    pub repeatable: bool,
    pub locations: Vec<DirectiveLocation>,
    pub span: Span,
}

#[derive(Clone, Copy)]
pub enum DirectiveLocation {
    Query,
    Mutation,
    Subscription,
    Field,
    FragmentDefinition,
    FragmentSpread,
    InlineFragment,
    Schema,
    Scalar,
    Object,
    FieldDefinition,
    ArgumentDefinition,
    Interface,
    Union,
    Enum,
    EnumValue,
    InputObject,
    InputFieldDefinition,
    VariableDefinition,
}

impl FromStr for DirectiveLocation {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "QUERY" => DirectiveLocation::Query,
            "MUTATION" => DirectiveLocation::Mutation,
            "SUBSCRIPTION" => DirectiveLocation::Subscription,
            "FIELD" => DirectiveLocation::Field,
            "FRAGMENT_DEFINITION" => DirectiveLocation::FragmentDefinition,
            "FRAGMENT_SPREAD" => DirectiveLocation::FragmentSpread,
            "INLINE_FRAGMENT" => DirectiveLocation::InlineFragment,
            "SCHEMA" => DirectiveLocation::Schema,
            "SCALAR" => DirectiveLocation::Scalar,
            "OBJECT" => DirectiveLocation::Object,
            "FIELD_DEFINITION" => DirectiveLocation::FieldDefinition,
            "ARGUMENT_DEFINITION" => DirectiveLocation::ArgumentDefinition,
            "INTERFACE" => DirectiveLocation::Interface,
            "UNION" => DirectiveLocation::Union,
            "ENUM" => DirectiveLocation::Enum,
            "ENUM_VALUE" => DirectiveLocation::EnumValue,
            "INPUT_OBJECT" => DirectiveLocation::InputObject,
            "INPUT_FIELD_DEFINITION" => DirectiveLocation::InputFieldDefinition,
            "VARIABLE_DEFINITION" => DirectiveLocation::VariableDefinition,
            _ => return Err(()),
        })
    }
}

impl std::fmt::Display for DirectiveLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            DirectiveLocation::Query => "QUERY",
            DirectiveLocation::Mutation => "MUTATION",
            DirectiveLocation::Subscription => "SUBSCRIPTION",
            DirectiveLocation::Field => "FIELD",
            DirectiveLocation::FragmentDefinition => "FRAGMENT_DEFINITION",
            DirectiveLocation::FragmentSpread => "FRAGMENT_SPREAD",
            DirectiveLocation::InlineFragment => "INLINE_FRAGMENT",
            DirectiveLocation::Schema => "SCHEMA",
            DirectiveLocation::Scalar => "SCALAR",
            DirectiveLocation::Object => "OBJECT",
            DirectiveLocation::FieldDefinition => "FIELD_DEFINITION",
            DirectiveLocation::ArgumentDefinition => "ARGUMENT_DEFINITION",
            DirectiveLocation::Interface => "INTERFACE",
            DirectiveLocation::Union => "UNION",
            DirectiveLocation::Enum => "ENUM",
            DirectiveLocation::EnumValue => "ENUM_VALUE",
            DirectiveLocation::InputObject => "INPUT_OBJECT",
            DirectiveLocation::InputFieldDefinition => "INPUT_FIELD_DEFINITION",
            DirectiveLocation::VariableDefinition => "VARIABLE_DEFINITION",
        };
        write!(f, "{str}")
    }
}

pub struct RootOperationTypeDefinition {
    pub operation_type: OperationType,
    pub named_type: StringId,
}

pub struct Type {
    pub name: StringId,
    pub wrappers: Vec<WrappingType>,
}

pub enum WrappingType {
    NonNull,
    List,
}

#[derive(Clone, Copy, Debug)]
pub enum OperationType {
    Query,
    Mutation,
    Subscription,
}

impl std::fmt::Display for OperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperationType::Query => write!(f, "query"),
            OperationType::Mutation => write!(f, "mutation"),
            OperationType::Subscription => write!(f, "subscription"),
        }
    }
}

pub enum StringLiteral {
    Normal(StringId),
    Block(StringId),
}

pub struct Directive {
    pub name: StringId,
    pub arguments: Vec<ArgumentId>,
}

pub struct Argument {
    pub name: StringId,
    pub value: ValueId,
}

pub enum Value {
    Variable(StringId),
    Int(i32),
    Float(f32),
    String(StringId),
    Boolean(bool),
    Null,
    Enum(StringId),
    List(Vec<ValueId>),
    Object(Vec<(StringId, ValueId)>),
}

#[derive(Default)]
pub struct AstBuilder {
    ast: Ast,
}

// TODO: Don't forget the spans etc.
impl AstBuilder {
    pub fn new() -> Self {
        AstBuilder {
            ast: Default::default(),
        }
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
