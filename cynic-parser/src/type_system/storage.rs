use crate::common::{IdRange, OperationType, TypeWrappers};

use super::{ids::*, DirectiveLocation, Span};

pub struct SchemaDefinition {
    pub description: Option<StringLiteralId>,
    pub roots: Vec<RootOperationTypeDefinition>,
}

pub struct ScalarDefinition {
    pub name: StringId,
    pub description: Option<StringLiteralId>,
    pub directives: IdRange<DirectiveId>,
    pub span: Span,
}

pub struct ObjectDefinition {
    pub name: StringId,
    pub description: Option<StringLiteralId>,
    pub fields: IdRange<FieldDefinitionId>,
    pub directives: IdRange<DirectiveId>,
    pub implements: Vec<StringId>,
    pub span: Span,
}

pub struct FieldDefinition {
    pub name: StringId,
    pub ty: TypeId,
    pub arguments: IdRange<InputValueDefinitionId>,
    pub description: Option<StringLiteralId>,
    pub directives: IdRange<DirectiveId>,
    pub span: Span,
}

pub struct InterfaceDefinition {
    pub name: StringId,
    pub description: Option<StringLiteralId>,
    pub fields: IdRange<FieldDefinitionId>,
    pub directives: IdRange<DirectiveId>,
    pub implements: Vec<StringId>,
    pub span: Span,
}

pub struct UnionDefinition {
    pub name: StringId,
    pub description: Option<StringLiteralId>,
    pub members: Vec<StringId>,
    pub directives: IdRange<DirectiveId>,
    pub span: Span,
}

pub struct EnumDefinition {
    pub name: StringId,
    pub description: Option<StringLiteralId>,
    pub values: Vec<EnumValueDefinitionId>,
    pub directives: IdRange<DirectiveId>,
    pub span: Span,
}

pub struct EnumValueDefinition {
    pub value: StringId,
    pub description: Option<StringLiteralId>,
    pub directives: IdRange<DirectiveId>,
    pub span: Span,
}

pub struct InputObjectDefinition {
    pub name: StringId,
    pub description: Option<StringLiteralId>,
    pub fields: IdRange<InputValueDefinitionId>,
    pub directives: IdRange<DirectiveId>,
    pub span: Span,
}

pub struct InputValueDefinition {
    pub name: StringId,
    pub ty: TypeId,
    pub description: Option<StringLiteralId>,
    pub default: Option<ValueId>,
    pub directives: IdRange<DirectiveId>,
    pub span: Span,
}

pub struct DirectiveDefinition {
    pub name: StringId,
    pub description: Option<StringLiteralId>,
    pub arguments: IdRange<InputValueDefinitionId>,
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
    pub wrappers: TypeWrappers,
}

pub struct Directive {
    pub name: StringId,
    pub arguments: Vec<ArgumentId>,
}

pub struct Argument {
    pub name: StringId,
    pub value: ValueId,
}

// TODO: This is the type_system value so it should maybe be
// ConstValue

pub enum ValueRecord {
    Variable(StringId),
    Int(i32),
    Float(f32),
    String(StringId),
    BlockString(BlockStringLiteralId),
    Boolean(bool),
    Null,
    Enum(StringId),
    List(Vec<ValueId>),
    Object(Vec<(StringId, ValueId)>),
}
