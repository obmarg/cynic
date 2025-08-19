//! Handles "aligning" rust types with the underlying schema types.
//!
//! We need to do this in certain cases where GraphQLs type coercion rules
//! don't quite match up with how rust works. For example: output fields
//! that are optional in rust but required in graphql - this is allowed,
//! but we need to do a bit of work for rust to be ok with it.

use std::borrow::Cow;

use syn::parse_quote;

use crate::schema::types::{InputType, OutputType, TypeRef};

use super::parsing::{parse_rust_type, RustType};

pub fn align_output_type(ty: &syn::Type, gql_ty: &TypeRef<'_, OutputType<'_>>) -> syn::Type {
    align_output_type_impl(&parse_rust_type(ty), gql_ty).to_syn()
}

fn align_output_type_impl<'a>(
    ty: &RustType<'a>,
    gql_ty: &TypeRef<'_, OutputType<'_>>,
) -> RustType<'a> {
    match (&ty, &gql_ty) {
        (RustType::SimpleType { .. }, _) => ty.clone(),
        (RustType::Optional { inner, .. }, TypeRef::Named(_, _) | TypeRef::List(_)) => {
            // If the rust type is optional but the schema type isn't
            // then we just ignore the `Option<_>` and recurse
            align_output_type_impl(inner.as_ref(), gql_ty)
        }
        (RustType::Optional { inner, .. }, crate::schema::types::TypeRef::Nullable(inner_gql)) => {
            // This is fine, but we may still need to align the inner types.
            let new_inner = align_output_type_impl(inner.as_ref(), inner_gql.as_ref());
            ty.clone().replace_inner(new_inner)
        }

        (RustType::List { inner, .. }, crate::schema::types::TypeRef::List(inner_gql)) => {
            // This is fine, but we may still need to align the inner types.
            let new_inner = align_output_type_impl(inner.as_ref(), inner_gql.as_ref());
            ty.clone().replace_inner(new_inner)
        }
        (RustType::List { .. }, _) => {
            // Something weird is up if we hit this path so don't mess with anything.
            ty.clone()
        }
        (RustType::Ref { inner, .. }, _) => {
            // This is fine, but we may still need to align the inner types.
            let new_inner = align_output_type_impl(inner.as_ref(), gql_ty);
            ty.clone().replace_inner(new_inner)
        }
    }
}

pub fn align_defaulted_output_type(
    ty: &syn::Type,
    gql_ty: &TypeRef<'_, OutputType<'_>>,
) -> syn::Type {
    let inner = align_output_type(ty, gql_ty);
    parse_quote! { Option<#inner> }
}

pub fn align_input_type(
    ty: &syn::Type,
    gql_ty: &TypeRef<'_, InputType<'_>>,
    gql_field_has_default: bool,
) -> syn::Type {
    align_input_type_impl(&parse_rust_type(ty), gql_ty, gql_field_has_default).to_syn()
}

fn align_input_type_impl<'a>(
    ty: &RustType<'a>,
    gql_ty: &TypeRef<'_, InputType<'_>>,
    gql_field_has_default: bool,
) -> RustType<'a> {
    match (&ty, &gql_ty) {
        (RustType::Ref { inner, syn, span }, _) => {
            // We need to be careful with lifetimes - if we're doing any additional wrapping we need to
            // make sure that the reference goes _inside_ the wrapper, as otherwise we might end up
            // putting an unsized type inside an Option.  e.g. `&'a Option<str>` which won't compile...
            match (inner.as_ref(), gql_ty) {
                (RustType::SimpleType { .. }, TypeRef::List(_)) => {
                    let syn = Cow::Owned(parse_quote! { ::std::vec::Vec<#syn> });
                    let wrapped_rust_type = RustType::List {
                        syn,
                        inner: Box::new(ty.clone()),
                        span: *span,
                    };
                    align_input_type_impl(&wrapped_rust_type, gql_ty, false)
                }
                (RustType::SimpleType { .. }, TypeRef::Nullable(_)) => {
                    let syn = Cow::Owned(parse_quote! { ::core::option::Option<#syn> });
                    let wrapped_rust_type = RustType::Optional {
                        syn,
                        inner: Box::new(ty.clone()),
                        span: *span,
                    };
                    align_input_type_impl(&wrapped_rust_type, gql_ty, false)
                }
                _ => {
                    // Transform the inner types, preserving the reference.
                    let new_inner =
                        align_input_type_impl(inner.as_ref(), gql_ty, gql_field_has_default);
                    ty.clone().replace_inner(new_inner)
                }
            }
        }

        (RustType::List { inner, .. }, TypeRef::List(inner_gql)) => {
            // Transform the inner types
            let new_inner = align_input_type_impl(inner.as_ref(), inner_gql, false);
            ty.clone().replace_inner(new_inner)
        }
        (RustType::Optional { inner, .. }, TypeRef::Nullable(inner_gql)) => {
            // Transform the inner types
            let new_inner = align_input_type_impl(inner.as_ref(), inner_gql, false);
            ty.clone().replace_inner(new_inner)
        }
        (RustType::Optional { inner, .. }, _) if gql_field_has_default => {
            // If GQL field has default then we ignore the Option
            align_input_type_impl(inner.as_ref(), gql_ty, false)
        }
        (other, TypeRef::List(_)) => {
            // Wrap our rust type in a vec, then recurse
            let syn = other.to_syn();
            let parsed = parse_quote! { ::std::vec::Vec<#syn> };
            align_input_type_impl(&parse_rust_type(&parsed), gql_ty, false).into_owned()
        }
        (other, TypeRef::Nullable(_)) => {
            // Wrap our rust named type in an option, then recurse
            let syn = other.to_syn();
            let parsed = parse_quote! { ::core::option::Option<#syn> };
            align_input_type_impl(&parse_rust_type(&parsed), gql_ty, false).into_owned()
        }
        (RustType::SimpleType { .. }, _) => ty.clone(),
        (RustType::Optional { .. } | RustType::List { .. }, _) => {
            // Something weird is up if we hit this path so don't mess with anything.
            ty.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use {proc_macro2::TokenStream, quote::quote, rstest::rstest, syn::parse2};

    use crate::schema::types::TypeRef;

    use super::*;

    #[rstest]
    #[case::additional_outer_option(
        quote! { Option<Vec<i32>> },
        list(integer()),
        quote! { Vec<i32> }
    )]
    #[case::additional_inner_option(
        quote! { Vec<Option<i32>> },
        list(integer()),
        quote! { Vec<i32> }
    )]
    #[case::additional_inner_and_outer_option(
        quote! { Option<Vec<Option<i32>>> },
        list(integer()),
        quote! { Vec<i32> }
    )]
    #[case::additional_inner_option_expected_outer(
        quote! { Option<Vec<Option<i32>>> },
        nullable(list(integer())),
        quote! { Option<Vec<i32>> }
    )]
    #[case::additional_outer_option_expected_inner(
        quote! { Option<Vec<Option<i32>>> },
        list(nullable(integer())),
        quote! { Vec<Option<i32>> }
    )]
    fn test_align_output_type(
        #[case] rust_type: TokenStream,
        #[case] graphql_type: TypeRef<'_, OutputType<'_>>,
        #[case] aligned_type: TokenStream,
    ) {
        let rust_type = parse2(rust_type).unwrap();
        let expected = parse2::<syn::Type>(aligned_type).unwrap();

        let result = align_output_type(&rust_type, &graphql_type);

        let expected_quote = quote! { #expected };
        let result_quote = quote! { #result };

        assert_eq!(
            expected, result,
            "Expected {expected_quote} got {result_quote}"
        );
    }

    #[rstest]
    #[case::an_integer(
        quote! { i32 },
        integer(),
    )]
    #[case::a_list(
        quote! { Vec<i32> },
        list(integer()),
    )]
    #[case::nested_list(
        quote! { Vec<Vec<i32>> },
        list(list(integer())),
    )]
    #[case::optional_list(
        quote! { Option<Vec<i32>> },
        nullable(list(integer())),
    )]
    #[case::list_containing_optional(
        quote! { Vec<Option<i32>> },
        list(nullable(integer())),
    )]
    #[case::optional_list_options(
        quote! { Option<Vec<Option<i32>>> },
        nullable(list(nullable(integer()))),
    )]
    fn test_align_output_type_with_no_changes(
        #[case] rust_type: TokenStream,
        #[case] graphql_type: TypeRef<'_, OutputType<'_>>,
    ) {
        let input = parse2::<syn::Type>(rust_type).unwrap();

        let result = align_output_type(&input, &graphql_type);

        let input_quote = quote! { #input };
        let result_quote = quote! { #result };

        assert_eq!(input, result, "Expected {input_quote} got {result_quote}");
    }

    #[rstest]
    #[case::missing_outer_option(
        quote! { Vec<i32> },
        nullable(list(integer())),
        false,
        quote! { ::core::option::Option<Vec<i32>> }
    )]
    #[case::missing_inner_option(
        quote! { Vec<i32> },
        list(nullable(integer())),
        false,
        quote! { Vec<::core::option::Option<i32>> }
    )]
    #[case::missing_inner_and_outer_option(
        quote! { Vec<i32> },
        nullable(list(nullable(integer()))),
        false,
        quote! { ::core::option::Option<Vec<::core::option::Option<i32>>> },
    )]
    #[case::missing_vec(
        quote! { i32 },
        list(integer()),
        false,
        quote! { ::std::vec::Vec<i32> }
    )]
    #[case::missing_vec_with_option(
        quote! { i32 },
        list(nullable(integer())),
        false,
        quote! { ::std::vec::Vec<::core::option::Option<i32>> }
    )]
    #[case::missing_optional_vec(
        quote! { i32 },
        nullable(list(integer())),
        false,
        quote! { ::core::option::Option<::std::vec::Vec<i32>> }
    )]
    #[case::missing_option_vec_option(
        quote! { i32 },
        nullable(list(nullable(integer()))),
        false,
        quote! { ::core::option::Option<::std::vec::Vec<::core::option::Option<i32>>> }
    )]
    #[case::default_on_non_nullable_scalar(
        quote! { Option<i32> },
        integer(),
        true,
        quote! { i32 }
    )]
    #[case::default_on_non_nullable_list(
        quote! { Option<Vec<i32>> },
        list(integer()),
        true,
        quote! { Vec<i32> }
    )]
    fn test_align_input_type(
        #[case] rust_type: TokenStream,
        #[case] graphql_type: TypeRef<'_, InputType<'_>>,
        #[case] graphql_type_is_optional: bool,
        #[case] aligned_type: TokenStream,
    ) {
        let rust_type = parse2(rust_type).unwrap();
        let expected = parse2::<syn::Type>(aligned_type).unwrap();

        let result = align_input_type(&rust_type, &graphql_type, graphql_type_is_optional);

        let expected_quote = quote! { #expected };
        let result_quote = quote! { #result };

        assert_eq!(
            expected, result,
            "Expected {expected_quote} got {result_quote}"
        );
    }

    #[rstest]
    #[case::an_integer(
        quote! { i32 },
        integer(),
        false,
    )]
    #[case::a_list(
        quote! { Vec<i32> },
        list(integer()),
        false
    )]
    #[case::nested_list(
        quote! { Vec<Vec<i32>> },
        list(list(integer())),
        false
    )]
    #[case::optional_list(
        quote! { Option<Vec<i32>> },
        nullable(list(integer())),
        false
    )]
    #[case::list_containing_optional(
        quote! { Vec<Option<i32>> },
        list(nullable(integer())),
        false
    )]
    #[case::optional_list_options(
        quote! { Option<Vec<Option<i32>>> },
        nullable(list(nullable(integer()))),
        false
    )]
    #[case::an_integer_with_default(
        quote! { i32 },
        integer(),
        true,
    )]
    #[case::a_list_with_default(
        quote! { Vec<i32> },
        list(integer()),
        true
    )]
    #[case::nested_list_with_default(
        quote! { Vec<Vec<i32>> },
        list(list(integer())),
        true
    )]
    #[case::optional_list_with_default(
        quote! { Option<Vec<i32>> },
        nullable(list(integer())),
        true
    )]
    #[case::list_containing_optional_with_default(
        quote! { Vec<Option<i32>> },
        list(nullable(integer())),
        true
    )]
    #[case::optional_list_options_with_default(
        quote! { Option<Vec<Option<i32>>> },
        nullable(list(nullable(integer()))),
        true
    )]
    fn test_align_input_type_with_no_changes(
        #[case] rust_type: TokenStream,
        #[case] graphql_type: TypeRef<'_, InputType<'_>>,
        #[case] graphql_type_is_optional: bool,
    ) {
        let input = parse2(rust_type).unwrap();

        let result = align_input_type(&input, &graphql_type, graphql_type_is_optional);

        let input_quote = quote! { #input };
        let result_quote = quote! { #result };

        assert_eq!(input, result, "expected {input_quote} got {result_quote}");
    }

    #[test]
    fn test_align_reference_types() {
        let input = parse2(quote! { &'a str }).unwrap();

        let result = align_input_type(&input, &nullable(string()), true);

        let expected = quote! { ::core::option::Option<&'a str> }.to_string();
        let result = quote! { #result }.to_string();

        assert_eq!(expected, result, "expected {expected} got {result}");
    }

    #[test]
    fn test_align_double_nested_reference_types() {
        let input = parse2(quote! { &'a str }).unwrap();

        let result = align_input_type(&input, &list(nullable(string())), true);

        let result = quote! { #result }.to_string();
        let expected = quote! { ::std::vec::Vec<::core::option::Option<&'a str> > }.to_string();

        assert_eq!(expected, result, "expected {expected} got {result}");
    }

    fn integer<'a, Kind>() -> TypeRef<'a, Kind> {
        TypeRef::Named("Int".into(), PhantomData)
    }

    fn string<'a, Kind>() -> TypeRef<'a, Kind> {
        TypeRef::Named("String".into(), PhantomData)
    }

    fn list<Kind>(inner: TypeRef<'_, Kind>) -> TypeRef<'_, Kind> {
        TypeRef::List(Box::new(inner))
    }

    fn nullable<Kind>(inner: TypeRef<'_, Kind>) -> TypeRef<'_, Kind> {
        TypeRef::Nullable(Box::new(inner))
    }
}
