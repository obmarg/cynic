use super::{ids::*, DirectiveLocation, OperationType, Span, WrappingType};

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

/// GraphQL wrappers encoded into a single u32
///
/// Bit 0: Whether the inner type is null
/// Bits 1..5: Number of list wrappers
/// Bits 5..21: List wrappers, where 0 is nullable 1 is non-null
/// The rest: dead bits
#[derive(Debug)]
pub struct TypeWrappers(u32);

static INNER_NULLABILITY_MASK: u32 = 1;
static NUM_LISTS_MASK: u32 = 32 - 2;
static NON_NUM_LISTS_MASK: u32 = u32::MAX ^ NUM_LISTS_MASK;

impl TypeWrappers {
    pub fn none() -> Self {
        TypeWrappers(0)
    }

    pub fn wrap_list(&self) -> Self {
        let current_wrappers = self.num_list_wrappers();

        let new_wrappers = current_wrappers + 1;
        assert!(new_wrappers < 16, "list wrapper overflow");

        Self((new_wrappers << 1) | (self.0 & NON_NUM_LISTS_MASK))
    }

    pub fn wrap_non_null(&self) -> Self {
        let index = self.num_list_wrappers();
        if index == 0 {
            return Self(INNER_NULLABILITY_MASK);
        }

        let new = self.0 | (1 << (4 + index));

        TypeWrappers(new)
    }

    pub fn iter(&self) -> TypeWrappersIter {
        let current_wrappers = self.num_list_wrappers();
        TypeWrappersIter {
            encoded: self.0,
            mask: (1 << (4 + current_wrappers)),
            next: None,
            last: ((INNER_NULLABILITY_MASK & self.0) == INNER_NULLABILITY_MASK)
                .then_some(WrappingType::NonNull),
        }
    }

    fn num_list_wrappers(&self) -> u32 {
        (self.0 & NUM_LISTS_MASK) >> 1
    }
}

impl FromIterator<WrappingType> for TypeWrappers {
    fn from_iter<T: IntoIterator<Item = WrappingType>>(iter: T) -> Self {
        iter.into_iter()
            .fold(TypeWrappers::none(), |wrappers, wrapping| match wrapping {
                WrappingType::NonNull => wrappers.wrap_non_null(),
                WrappingType::List => wrappers.wrap_list(),
            })
    }
}

pub struct TypeWrappersIter {
    encoded: u32,
    mask: u32,
    next: Option<WrappingType>,
    last: Option<WrappingType>,
}

impl Iterator for TypeWrappersIter {
    type Item = WrappingType;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.next.take() {
            return Some(next);
        }
        if (self.mask & NUM_LISTS_MASK) != 0 {
            if let Some(last) = self.last.take() {
                return Some(last);
            }
            return None;
        }

        // Otherwise we still have list wrappers
        let current_is_non_null = (self.encoded & self.mask) != 0;
        self.mask >>= 1;

        if current_is_non_null {
            self.next = Some(WrappingType::List);
            Some(WrappingType::NonNull)
        } else {
            Some(WrappingType::List)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{storage::TypeWrappers, WrappingType};

    #[test]
    fn test_wrappers() {
        assert_eq!(TypeWrappers::none().iter().collect::<Vec<_>>(), vec![]);
        assert_eq!(
            TypeWrappers::none()
                .wrap_non_null()
                .iter()
                .collect::<Vec<_>>(),
            vec![WrappingType::NonNull]
        );

        assert_eq!(
            TypeWrappers::none().wrap_list().iter().collect::<Vec<_>>(),
            vec![WrappingType::List]
        );

        assert_eq!(
            TypeWrappers::none()
                .wrap_non_null()
                .wrap_list()
                .iter()
                .collect::<Vec<_>>(),
            vec![WrappingType::List, WrappingType::NonNull]
        );

        assert_eq!(
            TypeWrappers::none()
                .wrap_non_null()
                .wrap_list()
                .wrap_non_null()
                .iter()
                .collect::<Vec<_>>(),
            vec![
                WrappingType::NonNull,
                WrappingType::List,
                WrappingType::NonNull
            ]
        );

        assert_eq!(
            TypeWrappers::none()
                .wrap_list()
                .wrap_list()
                .wrap_list()
                .wrap_non_null()
                .iter()
                .collect::<Vec<_>>(),
            vec![
                WrappingType::NonNull,
                WrappingType::List,
                WrappingType::List,
                WrappingType::List,
            ]
        );

        assert_eq!(
            TypeWrappers::none()
                .wrap_non_null()
                .wrap_list()
                .wrap_non_null()
                .wrap_list()
                .iter()
                .collect::<Vec<_>>(),
            vec![
                WrappingType::List,
                WrappingType::NonNull,
                WrappingType::List,
                WrappingType::NonNull
            ]
        );
    }
}
