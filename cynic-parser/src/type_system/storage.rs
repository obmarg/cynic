use crate::common::{IdRange, OperationType, TypeWrappers};

use super::{ids::*, DirectiveLocation, Span};

pub struct SchemaDefinition {
    pub description: Option<StringId>,
    pub roots: Vec<RootOperationTypeDefinition>,
}

pub struct ScalarDefinition {
    pub name: StringId,
    pub description: Option<StringId>,
    pub directives: IdRange<DirectiveId>,
    pub span: Span,
}

pub struct ObjectDefinition {
    pub name: StringId,
    pub description: Option<StringId>,
    pub fields: IdRange<FieldDefinitionId>,
    pub directives: IdRange<DirectiveId>,
    pub implements: Vec<StringId>,
    pub span: Span,
}

pub struct FieldDefinition {
    pub name: StringId,
    pub ty: TypeId,
    pub arguments: IdRange<InputValueDefinitionId>,
    pub description: Option<StringId>,
    pub directives: IdRange<DirectiveId>,
    pub span: Span,
}

pub struct InterfaceDefinition {
    pub name: StringId,
    pub description: Option<StringId>,
    pub fields: IdRange<FieldDefinitionId>,
    pub directives: IdRange<DirectiveId>,
    pub implements: Vec<StringId>,
    pub span: Span,
}

pub struct UnionDefinition {
    pub name: StringId,
    pub description: Option<StringId>,
    pub members: Vec<StringId>,
    pub directives: IdRange<DirectiveId>,
    pub span: Span,
}

pub struct EnumDefinition {
    pub name: StringId,
    pub description: Option<StringId>,
    pub values: Vec<EnumValueDefinitionId>,
    pub directives: IdRange<DirectiveId>,
    pub span: Span,
}

pub struct EnumValueDefinition {
    pub value: StringId,
    pub description: Option<StringId>,
    pub directives: IdRange<DirectiveId>,
    pub span: Span,
}

pub struct InputObjectDefinition {
    pub name: StringId,
    pub description: Option<StringId>,
    pub fields: IdRange<InputValueDefinitionId>,
    pub directives: IdRange<DirectiveId>,
    pub span: Span,
}

pub struct InputValueDefinition {
    pub name: StringId,
    pub ty: TypeId,
    pub description: Option<StringId>,
    pub default: Option<ValueId>,
    pub directives: IdRange<DirectiveId>,
    pub span: Span,
}

pub struct DirectiveDefinition {
    pub name: StringId,
    pub description: Option<StringId>,
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

pub enum StringLiteral {
    String(Box<str>),
    Block(Box<str>),
}

// TODO: This is the type_system value so it should maybe be
// ConstValue

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
