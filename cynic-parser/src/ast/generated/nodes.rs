use crate::{
    ast::{support, AstChildren, AstNode, NameOwner},
    syntax::{
        SyntaxKind::{self, *},
        SyntaxNode, SyntaxToken,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Document {
    pub(crate) syntax: SyntaxNode,
}
impl Document {
    pub fn definitions(&self) -> AstChildren<ExecutableDef> {
        support::children(&self.syntax)
    }
}
impl AstNode for Document {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == DOCUMENT
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OperationDef {
    pub(crate) syntax: SyntaxNode,
}
impl OperationDef {
    pub fn directives(&self) -> Option<Directives> {
        support::child(&self.syntax)
    }
    pub fn operation_type(&self) -> Option<OperationType> {
        support::child(&self.syntax)
    }
    pub fn selection_set(&self) -> Option<SelectionSet> {
        support::child(&self.syntax)
    }
    pub fn variable_defs(&self) -> Option<VariableDefs> {
        support::child(&self.syntax)
    }
}
impl AstNode for OperationDef {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == OPERATION_DEF
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl NameOwner for OperationDef {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FragmentDef {
    pub(crate) syntax: SyntaxNode,
}
impl FragmentDef {
    pub fn directives(&self) -> Option<Directives> {
        support::child(&self.syntax)
    }
    pub fn fragment_keyword_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, FRAGMENT_KEYWORD)
    }
    pub fn fragment_name(&self) -> Option<FragmentName> {
        support::child(&self.syntax)
    }
    pub fn selection_set(&self) -> Option<SelectionSet> {
        support::child(&self.syntax)
    }
    pub fn type_condition(&self) -> Option<TypeCondition> {
        support::child(&self.syntax)
    }
}
impl AstNode for FragmentDef {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == FRAGMENT_DEF
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OperationType {
    pub(crate) syntax: SyntaxNode,
}
impl OperationType {
    pub fn mutation_keyword_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, MUTATION_KEYWORD)
    }
    pub fn query_keyword_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, QUERY_KEYWORD)
    }
    pub fn subscription_keyword_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, SUBSCRIPTION_KEYWORD)
    }
}
impl AstNode for OperationType {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == OPERATION_TYPE
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VariableDefs {
    pub(crate) syntax: SyntaxNode,
}
impl VariableDefs {
    pub fn variable_def(&self) -> AstChildren<VariableDef> {
        support::children(&self.syntax)
    }
}
impl AstNode for VariableDefs {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == VARIABLE_DEFS
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Directives {
    pub(crate) syntax: SyntaxNode,
}
impl Directives {
    pub fn directive(&self) -> AstChildren<Directive> {
        support::children(&self.syntax)
    }
}
impl AstNode for Directives {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == DIRECTIVES
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SelectionSet {
    pub(crate) syntax: SyntaxNode,
}
impl SelectionSet {
    pub fn selections(&self) -> AstChildren<Selection> {
        support::children(&self.syntax)
    }
    pub fn open_curly_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, OPEN_CURLY)
    }
    pub fn close_curly_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, CLOSE_CURLY)
    }
}
impl AstNode for SelectionSet {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SELECTION_SET
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FragmentName {
    pub(crate) syntax: SyntaxNode,
}
impl FragmentName {}
impl AstNode for FragmentName {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == FRAGMENT_NAME
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl NameOwner for FragmentName {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeCondition {
    pub(crate) syntax: SyntaxNode,
}
impl TypeCondition {
    pub fn named_type(&self) -> Option<NamedType> {
        support::child(&self.syntax)
    }
    pub fn on_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, ON)
    }
}
impl AstNode for TypeCondition {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == TYPE_CONDITION
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VariableDef {
    pub(crate) syntax: SyntaxNode,
}
impl VariableDef {
    pub fn colon_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, COLON)
    }
    pub fn default_value(&self) -> Option<DefaultValue> {
        support::child(&self.syntax)
    }
    pub fn ty(&self) -> Option<Type> {
        support::child(&self.syntax)
    }
    pub fn variable(&self) -> Option<Variable> {
        support::child(&self.syntax)
    }
}
impl AstNode for VariableDef {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == VARIABLE_DEF
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Variable {
    pub(crate) syntax: SyntaxNode,
}
impl Variable {
    pub fn dollar_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, DOLLAR)
    }
}
impl AstNode for Variable {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == VARIABLE
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl NameOwner for Variable {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Type {
    pub(crate) syntax: SyntaxNode,
}
impl Type {
    pub fn bang_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, BANG)
    }
    pub fn open_square_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, OPEN_SQUARE)
    }
    pub fn close_square_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, CLOSE_SQUARE)
    }
    pub fn ty(&self) -> Option<Type> {
        support::child(&self.syntax)
    }
}
impl AstNode for Type {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == TYPE
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl NameOwner for Type {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DefaultValue {
    pub(crate) syntax: SyntaxNode,
}
impl DefaultValue {
    pub fn equals_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, EQUALS)
    }
    pub fn value(&self) -> Option<Value> {
        support::child(&self.syntax)
    }
}
impl AstNode for DefaultValue {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == DEFAULT_VALUE
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Directive {
    pub(crate) syntax: SyntaxNode,
}
impl Directive {
    pub fn at_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, AT)
    }
    pub fn arguments(&self) -> Option<Arguments> {
        support::child(&self.syntax)
    }
}
impl AstNode for Directive {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == DIRECTIVE
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl NameOwner for Directive {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Arguments {
    pub(crate) syntax: SyntaxNode,
}
impl Arguments {
    pub fn open_paren_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, OPEN_PAREN)
    }
    pub fn close_paren_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, CLOSE_PAREN)
    }
    pub fn argument(&self) -> AstChildren<Argument> {
        support::children(&self.syntax)
    }
}
impl AstNode for Arguments {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == ARGUMENTS
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Argument {
    pub(crate) syntax: SyntaxNode,
}
impl Argument {
    pub fn comma_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, COMMA)
    }
    pub fn colon_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, COLON)
    }
    pub fn value(&self) -> Option<Value> {
        support::child(&self.syntax)
    }
}
impl AstNode for Argument {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == ARGUMENT
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl NameOwner for Argument {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FloatValue {
    pub(crate) syntax: SyntaxNode,
}
impl FloatValue {
    pub fn negative_sign_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, NEGATIVE_SIGN)
    }
    pub fn exponent_part_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, EXPONENT_PART)
    }
    pub fn fractional_part_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, FRACTIONAL_PART)
    }
    pub fn number_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, NUMBER)
    }
}
impl AstNode for FloatValue {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == FLOAT_VALUE
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IntegerValue {
    pub(crate) syntax: SyntaxNode,
}
impl IntegerValue {
    pub fn negative_sign_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, NEGATIVE_SIGN)
    }
    pub fn number_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, NUMBER)
    }
}
impl AstNode for IntegerValue {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == INTEGER_VALUE
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StringValue {
    pub(crate) syntax: SyntaxNode,
}
impl StringValue {
    pub fn quote_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, QUOTE)
    }
    pub fn block_quote_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, BLOCK_QUOTE)
    }
}
impl AstNode for StringValue {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == STRING_VALUE
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectValue {
    pub(crate) syntax: SyntaxNode,
}
impl ObjectValue {
    pub fn object_field(&self) -> AstChildren<ObjectField> {
        support::children(&self.syntax)
    }
    pub fn open_curly_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, OPEN_CURLY)
    }
    pub fn close_curly_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, CLOSE_CURLY)
    }
}
impl AstNode for ObjectValue {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == OBJECT_VALUE
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ListValue {
    pub(crate) syntax: SyntaxNode,
}
impl ListValue {
    pub fn open_square_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, OPEN_SQUARE)
    }
    pub fn close_square_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, CLOSE_SQUARE)
    }
    pub fn value(&self) -> AstChildren<Value> {
        support::children(&self.syntax)
    }
}
impl AstNode for ListValue {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == LIST_VALUE
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BoolValue {
    pub(crate) syntax: SyntaxNode,
}
impl BoolValue {
    pub fn false_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, FALSE)
    }
    pub fn true_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, TRUE)
    }
}
impl AstNode for BoolValue {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == BOOL_VALUE
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Null {
    pub(crate) syntax: SyntaxNode,
}
impl Null {
    pub fn null_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, NULL)
    }
}
impl AstNode for Null {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == NULL
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnumValue {
    pub(crate) syntax: SyntaxNode,
}
impl EnumValue {}
impl AstNode for EnumValue {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == ENUM_VALUE
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl NameOwner for EnumValue {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectField {
    pub(crate) syntax: SyntaxNode,
}
impl ObjectField {
    pub fn colon_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, COLON)
    }
    pub fn value(&self) -> Option<Value> {
        support::child(&self.syntax)
    }
}
impl AstNode for ObjectField {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == OBJECT_FIELD
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl NameOwner for ObjectField {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FieldSelection {
    pub(crate) syntax: SyntaxNode,
}
impl FieldSelection {
    pub fn alias(&self) -> Option<Alias> {
        support::child(&self.syntax)
    }
    pub fn arguments(&self) -> Option<Arguments> {
        support::child(&self.syntax)
    }
    pub fn directives(&self) -> Option<Directives> {
        support::child(&self.syntax)
    }
    pub fn selection_set(&self) -> Option<SelectionSet> {
        support::child(&self.syntax)
    }
}
impl AstNode for FieldSelection {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == FIELD_SELECTION
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl NameOwner for FieldSelection {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InlineFragment {
    pub(crate) syntax: SyntaxNode,
}
impl InlineFragment {
    pub fn spread_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, SPREAD)
    }
    pub fn directives(&self) -> Option<Directives> {
        support::child(&self.syntax)
    }
    pub fn selection_set(&self) -> Option<SelectionSet> {
        support::child(&self.syntax)
    }
    pub fn type_condition(&self) -> Option<TypeCondition> {
        support::child(&self.syntax)
    }
}
impl AstNode for InlineFragment {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == INLINE_FRAGMENT
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FragmentSpread {
    pub(crate) syntax: SyntaxNode,
}
impl FragmentSpread {
    pub fn spread_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, SPREAD)
    }
    pub fn directives(&self) -> Option<Directives> {
        support::child(&self.syntax)
    }
    pub fn fragment_name(&self) -> Option<FragmentName> {
        support::child(&self.syntax)
    }
    pub fn selection_set(&self) -> Option<SelectionSet> {
        support::child(&self.syntax)
    }
}
impl AstNode for FragmentSpread {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == FRAGMENT_SPREAD
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Alias {
    pub(crate) syntax: SyntaxNode,
}
impl Alias {
    pub fn colon_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, COLON)
    }
}
impl AstNode for Alias {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == ALIAS
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl NameOwner for Alias {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NamedType {
    pub(crate) syntax: SyntaxNode,
}
impl NamedType {}
impl AstNode for NamedType {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == NAMED_TYPE
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl NameOwner for NamedType {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExecutableDef {
    OperationDef(OperationDef),
    FragmentDef(FragmentDef),
}
impl ExecutableDef {
    pub fn operation_def(&self) -> Option<OperationDef> {
        match self {
            Self::OperationDef(inner) => Some(inner.clone()),
            _ => None,
        }
    }
    pub fn is_operation_def(&self) -> bool {
        matches!(self, Self::OperationDef(_))
    }
    pub fn fragment_def(&self) -> Option<FragmentDef> {
        match self {
            Self::FragmentDef(inner) => Some(inner.clone()),
            _ => None,
        }
    }
    pub fn is_fragment_def(&self) -> bool {
        matches!(self, Self::FragmentDef(_))
    }
}
impl AstNode for ExecutableDef {
    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(kind, OPERATION_DEF | FRAGMENT_DEF)
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            OPERATION_DEF => ExecutableDef::OperationDef(OperationDef { syntax }),
            FRAGMENT_DEF => ExecutableDef::FragmentDef(FragmentDef { syntax }),
            _ => return None,
        };
        Some(res)
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            ExecutableDef::OperationDef(it) => &it.syntax,
            ExecutableDef::FragmentDef(it) => &it.syntax,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value {
    Variable(Variable),
    FloatValue(FloatValue),
    IntegerValue(IntegerValue),
    StringValue(StringValue),
    ObjectValue(ObjectValue),
    ListValue(ListValue),
    BoolValue(BoolValue),
    Null(Null),
    EnumValue(EnumValue),
}
impl Value {
    pub fn variable(&self) -> Option<Variable> {
        match self {
            Self::Variable(inner) => Some(inner.clone()),
            _ => None,
        }
    }
    pub fn is_variable(&self) -> bool {
        matches!(self, Self::Variable(_))
    }
    pub fn float_value(&self) -> Option<FloatValue> {
        match self {
            Self::FloatValue(inner) => Some(inner.clone()),
            _ => None,
        }
    }
    pub fn is_float_value(&self) -> bool {
        matches!(self, Self::FloatValue(_))
    }
    pub fn integer_value(&self) -> Option<IntegerValue> {
        match self {
            Self::IntegerValue(inner) => Some(inner.clone()),
            _ => None,
        }
    }
    pub fn is_integer_value(&self) -> bool {
        matches!(self, Self::IntegerValue(_))
    }
    pub fn string_value(&self) -> Option<StringValue> {
        match self {
            Self::StringValue(inner) => Some(inner.clone()),
            _ => None,
        }
    }
    pub fn is_string_value(&self) -> bool {
        matches!(self, Self::StringValue(_))
    }
    pub fn object_value(&self) -> Option<ObjectValue> {
        match self {
            Self::ObjectValue(inner) => Some(inner.clone()),
            _ => None,
        }
    }
    pub fn is_object_value(&self) -> bool {
        matches!(self, Self::ObjectValue(_))
    }
    pub fn list_value(&self) -> Option<ListValue> {
        match self {
            Self::ListValue(inner) => Some(inner.clone()),
            _ => None,
        }
    }
    pub fn is_list_value(&self) -> bool {
        matches!(self, Self::ListValue(_))
    }
    pub fn bool_value(&self) -> Option<BoolValue> {
        match self {
            Self::BoolValue(inner) => Some(inner.clone()),
            _ => None,
        }
    }
    pub fn is_bool_value(&self) -> bool {
        matches!(self, Self::BoolValue(_))
    }
    pub fn null(&self) -> Option<Null> {
        match self {
            Self::Null(inner) => Some(inner.clone()),
            _ => None,
        }
    }
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null(_))
    }
    pub fn enum_value(&self) -> Option<EnumValue> {
        match self {
            Self::EnumValue(inner) => Some(inner.clone()),
            _ => None,
        }
    }
    pub fn is_enum_value(&self) -> bool {
        matches!(self, Self::EnumValue(_))
    }
}
impl AstNode for Value {
    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(
            kind,
            VARIABLE
                | FLOAT_VALUE
                | INTEGER_VALUE
                | STRING_VALUE
                | OBJECT_VALUE
                | LIST_VALUE
                | BOOL_VALUE
                | NULL
                | ENUM_VALUE
        )
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            VARIABLE => Value::Variable(Variable { syntax }),
            FLOAT_VALUE => Value::FloatValue(FloatValue { syntax }),
            INTEGER_VALUE => Value::IntegerValue(IntegerValue { syntax }),
            STRING_VALUE => Value::StringValue(StringValue { syntax }),
            OBJECT_VALUE => Value::ObjectValue(ObjectValue { syntax }),
            LIST_VALUE => Value::ListValue(ListValue { syntax }),
            BOOL_VALUE => Value::BoolValue(BoolValue { syntax }),
            NULL => Value::Null(Null { syntax }),
            ENUM_VALUE => Value::EnumValue(EnumValue { syntax }),
            _ => return None,
        };
        Some(res)
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Value::Variable(it) => &it.syntax,
            Value::FloatValue(it) => &it.syntax,
            Value::IntegerValue(it) => &it.syntax,
            Value::StringValue(it) => &it.syntax,
            Value::ObjectValue(it) => &it.syntax,
            Value::ListValue(it) => &it.syntax,
            Value::BoolValue(it) => &it.syntax,
            Value::Null(it) => &it.syntax,
            Value::EnumValue(it) => &it.syntax,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Selection {
    FieldSelection(FieldSelection),
    InlineFragment(InlineFragment),
    FragmentSpread(FragmentSpread),
}
impl Selection {
    pub fn field_selection(&self) -> Option<FieldSelection> {
        match self {
            Self::FieldSelection(inner) => Some(inner.clone()),
            _ => None,
        }
    }
    pub fn is_field_selection(&self) -> bool {
        matches!(self, Self::FieldSelection(_))
    }
    pub fn inline_fragment(&self) -> Option<InlineFragment> {
        match self {
            Self::InlineFragment(inner) => Some(inner.clone()),
            _ => None,
        }
    }
    pub fn is_inline_fragment(&self) -> bool {
        matches!(self, Self::InlineFragment(_))
    }
    pub fn fragment_spread(&self) -> Option<FragmentSpread> {
        match self {
            Self::FragmentSpread(inner) => Some(inner.clone()),
            _ => None,
        }
    }
    pub fn is_fragment_spread(&self) -> bool {
        matches!(self, Self::FragmentSpread(_))
    }
}
impl AstNode for Selection {
    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(kind, FIELD_SELECTION | INLINE_FRAGMENT | FRAGMENT_SPREAD)
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            FIELD_SELECTION => Selection::FieldSelection(FieldSelection { syntax }),
            INLINE_FRAGMENT => Selection::InlineFragment(InlineFragment { syntax }),
            FRAGMENT_SPREAD => Selection::FragmentSpread(FragmentSpread { syntax }),
            _ => return None,
        };
        Some(res)
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Selection::FieldSelection(it) => &it.syntax,
            Selection::InlineFragment(it) => &it.syntax,
            Selection::FragmentSpread(it) => &it.syntax,
        }
    }
}