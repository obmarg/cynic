use std::borrow::Cow;

use proc_macro2::Span;
use syn::{spanned::Spanned, GenericArgument, TypePath};

/// A simplified rust type structure
#[derive(Debug, Clone)]
pub enum RustType<'a> {
    Optional {
        syn: Cow<'a, TypePath>,
        inner: Box<RustType<'a>>,
        span: Span,
    },
    List {
        syn: Cow<'a, TypePath>,
        inner: Box<RustType<'a>>,
        span: Span,
    },
    Box {
        syn: Cow<'a, TypePath>,
        inner: Box<RustType<'a>>,
        span: Span,
    },
    SimpleType {
        syn: Cow<'a, syn::Type>,
        span: Span,
    },
    Unknown {
        syn: Cow<'a, syn::Type>,
        span: Span,
    },
}

impl<'a> PartialEq for RustType<'a> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Optional {
                    syn: l_syn,
                    inner: l_inner,
                    ..
                },
                Self::Optional {
                    syn: r_syn,
                    inner: r_inner,
                    ..
                },
            ) => l_syn == r_syn && l_inner == r_inner,
            (
                Self::List {
                    syn: l_syn,
                    inner: l_inner,
                    ..
                },
                Self::List {
                    syn: r_syn,
                    inner: r_inner,
                    ..
                },
            ) => l_syn == r_syn && l_inner == r_inner,
            (
                Self::Box {
                    syn: l_syn,
                    inner: l_inner,
                    ..
                },
                Self::Box {
                    syn: r_syn,
                    inner: r_inner,
                    ..
                },
            ) => l_syn == r_syn && l_inner == r_inner,
            (Self::SimpleType { syn: l_syn, .. }, Self::SimpleType { syn: r_syn, .. }) => {
                l_syn == r_syn
            }
            (Self::Unknown { syn: l_syn, .. }, Self::Unknown { syn: r_syn, .. }) => l_syn == r_syn,
            _ => false,
        }
    }
}

impl<'a> RustType<'a> {
    pub fn convert_lifetime<'b>(self) -> RustType<'b> {
        match self {
            RustType::Optional { syn, inner, span } => RustType::Optional {
                syn: Cow::Owned(syn.into_owned()),
                inner: Box::new(inner.convert_lifetime()),
                span,
            },
            RustType::List { syn, inner, span } => RustType::List {
                syn: Cow::Owned(syn.into_owned()),
                inner: Box::new(inner.convert_lifetime()),
                span,
            },
            RustType::Box { syn, inner, span } => RustType::Box {
                syn: Cow::Owned(syn.into_owned()),
                inner: Box::new(inner.convert_lifetime()),
                span,
            },
            RustType::SimpleType { syn, span } => RustType::SimpleType {
                syn: Cow::Owned(syn.into_owned()),
                span,
            },
            RustType::Unknown { syn, span } => RustType::Unknown {
                syn: Cow::Owned(syn.into_owned()),
                span,
            },
        }
    }

    pub fn span(&self) -> Span {
        match self {
            RustType::Optional { span, .. } => *span,
            RustType::List { span, .. } => *span,
            RustType::Box { span, .. } => *span,
            RustType::SimpleType { span, .. } => *span,
            RustType::Unknown { span, .. } => *span,
        }
    }
}

#[allow(clippy::cmp_owned)]
pub fn parse_rust_type(ty: &'_ syn::Type) -> RustType<'_> {
    let span = ty.span();
    if let syn::Type::Path(type_path) = ty {
        if let Some(last_segment) = type_path.path.segments.last() {
            if let syn::PathArguments::None = last_segment.arguments {
                return RustType::SimpleType {
                    syn: Cow::Borrowed(ty),
                    span,
                };
            }

            match last_segment.ident.to_string().as_ref() {
                "Box" | "Arc" | "Rc" => {
                    if let Some(inner_type) = extract_generic_argument(last_segment) {
                        return RustType::Box {
                            syn: Cow::Borrowed(type_path),
                            inner: Box::new(parse_rust_type(inner_type)),
                            span,
                        };
                    }
                }
                "Option" => {
                    if let Some(inner_type) = extract_generic_argument(last_segment) {
                        return RustType::Optional {
                            syn: Cow::Borrowed(type_path),
                            inner: Box::new(parse_rust_type(inner_type)),
                            span,
                        };
                    }
                }
                "Vec" => {
                    if let Some(inner_type) = extract_generic_argument(last_segment) {
                        return RustType::List {
                            syn: Cow::Borrowed(type_path),
                            inner: Box::new(parse_rust_type(inner_type)),
                            span,
                        };
                    }
                }
                _ => {}
            }
            return RustType::Unknown {
                syn: Cow::Borrowed(ty),
                span,
            };
        }
    }

    RustType::Unknown {
        syn: Cow::Borrowed(ty),
        span,
    }
}

/// Takes a PathSegment like `Vec<T>` and extracts the `T`
fn extract_generic_argument(segment: &syn::PathSegment) -> Option<&syn::Type> {
    if let syn::PathArguments::AngleBracketed(angle_bracketed) = &segment.arguments {
        for arg in &angle_bracketed.args {
            if let syn::GenericArgument::Type(inner_type) = arg {
                return Some(inner_type);
            }
        }
    }

    None
}

impl<'a> RustType<'a> {
    pub fn to_syn(&self) -> syn::Type {
        match self {
            RustType::Optional { syn, .. } => syn::Type::Path(syn.clone().into_owned()),
            RustType::List { syn, .. } => syn::Type::Path(syn.clone().into_owned()),
            RustType::Box { syn, .. } => syn::Type::Path(syn.clone().into_owned()),
            RustType::SimpleType { syn, .. } => syn.clone().into_owned(),
            RustType::Unknown { syn, .. } => syn.clone().into_owned(),
        }
    }

    pub fn replace_inner(self, new_inner: RustType<'a>) -> RustType<'a> {
        match self {
            RustType::SimpleType { .. } | RustType::Unknown { .. } => {
                panic!("Can't replace inner on simple or unknown types")
            }
            RustType::Optional { mut syn, span, .. } => {
                syn.to_mut().replace_generic_param(&new_inner);
                RustType::Optional {
                    syn,
                    inner: Box::new(new_inner),
                    span,
                }
            }
            RustType::Box { mut syn, span, .. } => {
                syn.to_mut().replace_generic_param(&new_inner);
                RustType::Box {
                    syn,
                    inner: Box::new(new_inner),
                    span,
                }
            }
            RustType::List { mut syn, span, .. } => {
                syn.to_mut().replace_generic_param(&new_inner);
                RustType::List {
                    syn,
                    inner: Box::new(new_inner),
                    span,
                }
            }
        }
    }
}

trait TypePathExt {
    fn replace_generic_param(&mut self, replacement: &RustType<'_>);
}

impl TypePathExt for syn::TypePath {
    fn replace_generic_param(&mut self, replacement: &RustType<'_>) {
        fn get_generic_argument(type_path: &mut syn::TypePath) -> Option<&mut GenericArgument> {
            let segment = type_path.path.segments.last_mut()?;

            match &mut segment.arguments {
                syn::PathArguments::AngleBracketed(angle_bracketed) => {
                    angle_bracketed.args.first_mut()
                }
                _ => None,
            }
        }

        let generic_argument = get_generic_argument(self)
            .expect("Don't call replace_generic_param on a type without a generic argument");

        *generic_argument = syn::GenericArgument::Type(replacement.to_syn())
    }
}

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use quote::quote;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::replace_on_option(
        quote! { Option<i32> },
        quote! { Vec<i32> },
        quote! { Option<Vec<i32>> },
    )]
    #[case::replace_on_vec(
        quote! { Vec<i32> },
        quote! { Vec<i32> },
        quote! { Vec<Vec<i32>> },
    )]
    #[case::replace_on_box(
        quote! { Box<i32> },
        quote! { Vec<i32> },
        quote! { Box<Vec<i32>> },
    )]
    #[case::replace_on_arc(
        quote! { Arc<i32> },
        quote! { Vec<i32> },
        quote! { Arc<Vec<i32>> },
    )]
    #[case::replace_with_complex_inner(
        quote! { Arc<i32> },
        quote! { Vec<chrono::DateTime<chrono::Utc>> },
        quote! { Arc<Vec<chrono::DateTime<chrono::Utc>>> },
    )]
    #[case::replace_with_a_full_path(
        quote! { std::sync::Arc<i32> },
        quote! { Vec<chrono::DateTime<chrono::Utc>> },
        quote! { std::sync::Arc<Vec<chrono::DateTime<chrono::Utc>>> },
    )]
    fn test_replace_inner(
        #[case] original: TokenStream,
        #[case] replace: TokenStream,
        #[case] expected: TokenStream,
    ) {
        let original = syn::parse2(original).unwrap();
        let replace = syn::parse2(replace).unwrap();
        let expected = syn::parse2(expected).unwrap();

        let result = parse_rust_type(&original)
            .replace_inner(parse_rust_type(&replace))
            .to_syn();

        assert_eq!(result, expected);
    }
}
