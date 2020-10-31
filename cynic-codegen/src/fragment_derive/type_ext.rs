pub trait SynTypeExt {
    /// Extracts the inner type from a syn::Type.
    ///
    /// This takes some (potentially nested) Options & Vectors and extracts
    /// enough layers to get at the inner type.
    ///
    /// This is useful when deriving QueryFragment as we need to know which
    /// type to call `QueryFragment::fragment` on for any nested fields.
    fn inner_type(&self) -> syn::Type;
}

impl SynTypeExt for syn::Type {
    fn inner_type(&self) -> syn::Type {
        use syn::{GenericArgument, PathArguments, Type};

        if let Type::Path(expr) = self {
            if let Some(segment) = expr.path.segments.first() {
                let ident_string = segment.ident.to_string();
                if ident_string == "Option" || ident_string == "Vec" || ident_string == "Box" {
                    if let PathArguments::AngleBracketed(expr) = &segment.arguments {
                        if let Some(GenericArgument::Type(ty)) = expr.args.first() {
                            return ty.inner_type();
                        }
                    }
                }
            }
        }

        self.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::rstest;
    use syn::parse_quote;

    #[rstest(outer, inner,
        case(parse_quote!{ Option<Vec<Option<T>>> }, parse_quote!{ T }),
        case(parse_quote!{ Vec<Option<T>> }, parse_quote!{ T }),
        case(parse_quote!{ Option<T> }, parse_quote!{ T }),
        case(parse_quote!{ T }, parse_quote!{ T }),
    )]
    fn test_inner_type(outer: syn::Type, inner: syn::Type) {
        assert_eq!(outer.inner_type(), inner);
    }
}
