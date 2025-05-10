use std::borrow::Cow;

use {
    proc_macro2::Span,
    syn::{spanned::Spanned, GenericArgument, TypePath},
};

#[derive(Debug, Clone)]
pub enum RustType<'a> {
    Optional {
        syn: Cow<'a, TypePath>,
        inner: Box<RustType<'a>>,
        span: Span,
    },
    List {
        syn: Cow<'a, syn::Type>,
        inner: Box<RustType<'a>>,
        span: Span,
    },
    Ref {
        syn: Cow<'a, syn::Type>,
        inner: Box<RustType<'a>>,
        span: Span,
    },
    SimpleType {
        syn: Cow<'a, syn::Type>,
        span: Span,
    },
}

impl RustType<'_> {
    pub fn into_owned(self) -> RustType<'static> {
        match self {
            RustType::Optional { syn, inner, span } => RustType::Optional {
                syn: Cow::Owned(syn.into_owned()),
                inner: Box::new(inner.into_owned()),
                span,
            },
            RustType::List { syn, inner, span } => RustType::List {
                syn: Cow::Owned(syn.into_owned()),
                inner: Box::new(inner.into_owned()),
                span,
            },
            RustType::Ref { syn, inner, span } => RustType::Ref {
                syn: Cow::Owned(syn.into_owned()),
                inner: Box::new(inner.into_owned()),
                span,
            },
            RustType::SimpleType { syn, span } => RustType::SimpleType {
                syn: Cow::Owned(syn.into_owned()),
                span,
            },
        }
    }

    pub fn span(&self) -> Span {
        match self {
            RustType::Optional { span, .. } => *span,
            RustType::List { span, .. } => *span,
            RustType::Ref { span, .. } => *span,
            RustType::SimpleType { span, .. } => *span,
        }
    }
}

pub fn parse_rust_type(ty: &syn::Type) -> RustType<'_> {
    let span = ty.span();
    match ty {
        syn::Type::Path(type_path) => {
            if let Some(last_segment) = type_path.path.segments.last() {
                match last_segment.ident.to_string().as_str() {
                    "Box" | "Arc" | "Rc" => {
                        if let Some(inner_type) = extract_generic_argument(last_segment) {
                            return RustType::Ref {
                                syn: Cow::Borrowed(ty),
                                inner: Box::new(parse_rust_type(inner_type)),
                                span,
                            };
                        }
                    }
                    "MaybeUndefined" | "Option" => {
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
                                syn: Cow::Borrowed(ty),
                                inner: Box::new(parse_rust_type(inner_type)),
                                span,
                            };
                        }
                    }
                    _ => {}
                }
            }
        }
        syn::Type::Reference(syn::TypeReference { elem, .. })
            if matches!(**elem, syn::Type::Slice(_)) =>
        {
            let syn::Type::Slice(array) = &**elem else {
                unreachable!()
            };
            return RustType::List {
                syn: Cow::Borrowed(ty),
                inner: Box::new(parse_rust_type(&array.elem)),
                span,
            };
        }
        syn::Type::Reference(reference) => {
            return RustType::Ref {
                syn: Cow::Borrowed(ty),
                inner: Box::new(parse_rust_type(&reference.elem)),
                span,
            }
        }
        syn::Type::Array(array) => {
            return RustType::List {
                syn: Cow::Borrowed(ty),
                inner: Box::new(parse_rust_type(&array.elem)),
                span,
            }
        }
        syn::Type::Slice(slice) => {
            return RustType::List {
                syn: Cow::Borrowed(ty),
                inner: Box::new(parse_rust_type(&slice.elem)),
                span,
            }
        }
        _ => {}
    }

    RustType::SimpleType {
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
            RustType::List { syn, .. } => syn.clone().into_owned(),
            RustType::Ref { syn, .. } => syn.clone().into_owned(),
            RustType::SimpleType { syn, .. } => syn.clone().into_owned(),
        }
    }

    pub fn replace_inner(self, new_inner: RustType<'a>) -> RustType<'a> {
        match self {
            RustType::SimpleType { .. } => {
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
            RustType::Ref { mut syn, span, .. } => {
                match syn.to_mut() {
                    syn::Type::Path(path) => path.replace_generic_param(&new_inner),
                    syn::Type::Reference(reference) => reference.elem = Box::new(new_inner.to_syn()),
                    _ => panic!("We shouldn't have constructed RustType::Ref for anything else than these types")
                }
                RustType::Ref {
                    syn,
                    inner: Box::new(new_inner),
                    span,
                }
            }
            RustType::List { mut syn, span, .. } => {
                match syn.to_mut() {
                    syn::Type::Path(path) => path.replace_generic_param(&new_inner),
                    syn::Type::Array(array) => array.elem = Box::new(new_inner.to_syn()),
                    syn::Type::Slice(slice) => slice.elem = Box::new(new_inner.to_syn()),
                    syn::Type::Reference(ref_to_slice) => {
                        let syn::Type::Slice(slice) = &mut *ref_to_slice.elem
                            else { panic!("We shouldn't have constructed RustType::List for a Ref unless the type beneath is a Slice") };
                        slice.elem = Box::new(new_inner.to_syn());
                    }
                    _ => panic!("We shouldn't have constructed RustType::List for anything else than these types")
                }

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
    use {proc_macro2::TokenStream, quote::quote, rstest::rstest};

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
