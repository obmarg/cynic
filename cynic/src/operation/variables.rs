use crate::{variables::VariableType, QueryVariables};

pub struct VariableDefinitions {
    vars: &'static [(&'static str, VariableType)],
}

impl VariableDefinitions {
    pub fn new<T: QueryVariables>() -> Self {
        VariableDefinitions { vars: T::VARIABLES }
    }
}

impl std::fmt::Display for VariableDefinitions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.vars.is_empty() {
            return Ok(());
        }

        write!(f, "(")?;
        let mut first = true;
        for (name, ty) in self.vars {
            if !first {
                write!(f, ", ")?;
            }
            first = false;

            let ty = GraphqlVariableType::new(*ty);
            write!(f, "${name}: {ty}")?;
        }
        write!(f, ")")
    }
}

enum GraphqlVariableType {
    List(Box<GraphqlVariableType>),
    NotNull(Box<GraphqlVariableType>),
    Named(&'static str),
}

impl GraphqlVariableType {
    fn new(ty: VariableType) -> Self {
        fn recurse(ty: VariableType, required: bool) -> GraphqlVariableType {
            match (ty, required) {
                (VariableType::Nullable(inner), _) => recurse(*inner, false),
                (any, true) => GraphqlVariableType::NotNull(Box::new(recurse(any, false))),
                (VariableType::List(inner), _) => {
                    GraphqlVariableType::List(Box::new(recurse(*inner, true)))
                }
                (VariableType::Named(name), false) => GraphqlVariableType::Named(name),
            }
        }

        recurse(ty, true)
    }
}

impl std::fmt::Display for GraphqlVariableType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphqlVariableType::List(inner) => write!(f, "[{inner}]"),
            GraphqlVariableType::NotNull(inner) => write!(f, "{inner}!"),
            GraphqlVariableType::Named(name) => write!(f, "{name}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_printing() {
        insta::assert_display_snapshot!(VariableDefinitions {
            vars: &[
                ("foo", VariableType::List(&VariableType::Named("Foo"))),
                ("bar", VariableType::Named("Bar")),
                ("nullable_bar", VariableType::Nullable(&VariableType::Named("Bar"))),
                (
                    "nullable_list_foo",
                    VariableType::Nullable(&(VariableType::List(&VariableType::Named("Foo"))))
                ),
                (
                    "nullable_list_nullable_foo",
                    VariableType::Nullable(&VariableType::List(&VariableType::Nullable(
                        &VariableType::Named("Foo")
                    )))
                )
            ]
        }, @"($foo: [Foo!]!, $bar: Bar!, $nullable_bar: Bar, $nullable_list_foo: [Foo!], $nullable_list_nullable_foo: [Foo])")
    }
}
