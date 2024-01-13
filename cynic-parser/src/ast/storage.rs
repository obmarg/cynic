use super::{ids::*, DirectiveLocation, OperationType, Span, WrappingType};

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

pub struct RootOperationTypeDefinition {
    pub operation_type: OperationType,
    pub named_type: StringId,
}

pub struct Type {
    pub name: StringId,
    pub wrappers: Vec<WrappingType>,
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
