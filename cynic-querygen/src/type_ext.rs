use graphql_parser::query::Type;
use inflector::Inflector;
use std::borrow::Cow;

use crate::{schema::TypeDefinition, TypeIndex};

pub trait TypeExt<'a> {
    fn inner_name(&self) -> &str;
    fn type_spec(&self, type_index: &TypeIndex<'a>) -> Cow<'a, str>;
    fn is_required(&self) -> bool;
}

impl<'a> TypeExt<'a> for Type<'a, &'a str> {
    fn type_spec(&self, type_index: &TypeIndex<'a>) -> Cow<'a, str> {
        type_spec_imp(self, true, type_index)
    }

    fn inner_name(&self) -> &str {
        match self {
            Type::NamedType(s) => s,
            Type::ListType(inner) => inner.inner_name(),
            Type::NonNullType(inner) => inner.inner_name(),
        }
    }

    fn is_required(&self) -> bool {
        match self {
            Type::NonNullType(_) => true,
            _ => false,
        }
    }
}

fn type_spec_imp<'a>(
    ty: &Type<'a, &'a str>,
    nullable: bool,
    type_index: &TypeIndex<'a>,
) -> Cow<'a, str> {
    if let Type::NonNullType(inner) = ty {
        return type_spec_imp(inner, false, type_index);
    }

    if nullable {
        return Cow::Owned(format!("Option<{}>", type_spec_imp(ty, false, type_index)));
    }

    match ty {
        Type::ListType(inner) => {
            Cow::Owned(format!("Vec<{}>", type_spec_imp(inner, true, type_index)))
        }
        Type::NonNullType(_) => panic!("NonNullType somehow got past an if let"),
        Type::NamedType("Int") => Cow::Borrowed("i32"),
        Type::NamedType("Float") => Cow::Borrowed("f64"),
        Type::NamedType("Boolean") => Cow::Borrowed("bool"),
        Type::NamedType("ID") => Cow::Borrowed("cynic::Id"),
        Type::NamedType(s) => match type_index.lookup_type(s) {
            Some(TypeDefinition::Enum(_)) => Cow::Owned(s.to_pascal_case()),
            Some(TypeDefinition::Object(_)) => Cow::Owned(s.to_pascal_case()),
            _ => Cow::Borrowed(s),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest(
        input,
        expected,
        case(Type::NamedType("Int"), "Option<i32>"),
        case(Type::NamedType("Float"), "Option<f64>"),
        case(Type::NamedType("Boolean"), "Option<bool>"),
        case(Type::NamedType("ID"), "Option<cynic::Id>"),
        case(Type::ListType(Type::NamedType("ID").into()), "Option<Vec<Option<cynic::Id>>>"),
        case(Type::ListType(Type::NonNullType(Type::NamedType("ID").into()).into()), "Option<Vec<cynic::Id>>"),
        case(
            Type::NonNullType(Type::ListType(Type::NamedType("ID").into()).into()),
            "Vec<Option<cynic::Id>>"
        ),
        case(
            Type::NonNullType(Type::ListType(Type::NonNullType(Type::NamedType("ID").into()).into()).into()),
            "Vec<cynic::Id>"
        )
    )]
    fn type_spec_returns_correct_type(input: Type<'static, &'static str>, expected: &'static str) {
        let type_index = TypeIndex::default();
        assert_eq!(input.type_spec(&type_index), expected);
    }
}
