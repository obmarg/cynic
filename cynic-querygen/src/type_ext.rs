use graphql_parser::query::Type;
use std::borrow::Cow;

use crate::TypeIndex;

pub trait TypeExt<'a> {
    fn inner_name(&self) -> &str;
    fn type_spec(&self) -> Cow<'a, str>;
}

impl<'a> TypeExt<'a> for Type<'a, &'a str> {
    fn type_spec(&self, type_index: &TypeIndex<'a>) -> Cow<'a, str> {
        type_spec_imp(self, true)
    }

    fn inner_name(&self) -> &str {
        match self {
            Type::NamedType(s) => s,
            Type::ListType(inner) => inner.inner_name(),
            Type::NonNullType(inner) => inner.inner_name(),
        }
    }
}

fn type_spec_imp<'a>(
    ty: &Type<'a, &'a str>,
    nullable: bool,
    type_index: &TypeIndex<'a>,
) -> Cow<'a, str> {
    if let Type::NonNullType(inner) = ty {
        return type_spec_imp(inner, false);
    }

    if nullable {
        return Cow::Owned(format!("Option<{}>", type_spec_imp(ty, false)));
    }

    match ty {
        Type::ListType(inner) => Cow::Owned(format!("Vec<{}>", type_spec_imp(inner, true))),
        Type::NonNullType(inner) => panic!("NonNullType somehow got past an if let"),
        Type::NamedType("Int") => Cow::Borrowed("i64"),
        Type::NamedType("Float") => Cow::Borrowed("f64"),
        Type::NamedType("Boolean") => Cow::Borrowed("bool"),
        Type::NamedType(s) => {
            Cow::Borrowed(s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest(
        input,
        expected,
        case(Type::NamedType("ID"), "Option<ID>"),
        case(Type::ListType(Type::NamedType("ID").into()), "Option<Vec<Option<ID>>>"),
        case(Type::ListType(Type::NonNullType(Type::NamedType("ID").into()).into()), "Option<Vec<ID>>"),
        case(
            Type::NonNullType(Type::ListType(Type::NamedType("ID").into()).into()),
            "Vec<Option<ID>>"
        ),
        case(
            Type::NonNullType(Type::ListType(Type::NonNullType(Type::NamedType("ID").into()).into()).into()),
            "Vec<ID>"
        )
    )]
    fn type_spec_returns_correct_type(input: Type<'static, &'static str>, expected: &'static str) {
        assert_eq!(input.type_spec(), expected);
    }
}
