use graphql_parser::query::Type;

pub trait TypeExt<'a> {
    fn inner_name(&self) -> &'a str;
}

impl<'a> TypeExt<'a> for Type<'a, &'a str> {
    fn inner_name(&self) -> &'a str {
        match self {
            Type::NamedType(s) => s,
            Type::ListType(inner) => inner.inner_name(),
            Type::NonNullType(inner) => inner.inner_name(),
        }
    }
}
