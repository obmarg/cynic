// TODO: docs.

pub trait IntoArgument<Argument> {
    fn into_argument(self) -> Argument;
}

impl<T> IntoArgument<Option<T>> for T {
    fn into_argument(self) -> Option<T> {
        Some(self)
    }
}

// TODO: Can I take advantage of the fact that there's a limited
// subset of things that can be arguments here, and actually enumerate
// every possibility rather than adding this generic impl.
// This would give me a lot more leeway to do stuff with AsRef etc.

// Things that can be arguments: scalars, input types, vecs<other_args>, options<other_args>
// Actually very simple.
// Also worth noting that these are the only types that need to be serialized, and
// _also_ currently the only types SerializableArgument are implemented for...
impl<T> IntoArgument<T> for T {
    fn into_argument(self) -> T {
        self
    }
}

impl IntoArgument<String> for &str {
    fn into_argument(self) -> String {
        self.to_string()
    }
}

impl IntoArgument<Option<String>> for &str {
    fn into_argument(self) -> Option<String> {
        Some(self.to_string())
    }
}

// TODO: IntoArgument for Cow etc. ?

// TODO: Can I consolidate IntoArgument with Serializable argument somehow?
