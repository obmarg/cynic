//! Handles "aligning" rust types with the underlying schema types.
//!
//! We need to do this in certain cases where GraphQLs type coercion rules
//! don't quite match up with how rust works. For example: output fields
//! that are optional in rust but required in graphql - this is allowed,
//! but we need to do a bit of work for rust to be ok with it.

use crate::schema::types::{OutputType, TypeRef};

use super::parsing2::RustType;

pub fn align_output_type<'a>(
    ty: &RustType<'a>,
    gql_ty: &TypeRef<'_, OutputType<'_>>,
) -> RustType<'a> {
    match (&ty, &gql_ty) {
        (RustType::Unknown { .. } | RustType::SimpleType { .. }, _) => ty.clone(),
        (RustType::Optional { inner, .. }, TypeRef::Named(_, _, _) | TypeRef::List(_)) => {
            // If the rust type is optional but the schema type isn't
            // then we just ignore the `Option<_>` and recurse
            align_output_type(inner.as_ref(), gql_ty)
        }
        (RustType::Optional { inner, .. }, crate::schema::types::TypeRef::Nullable(inner_gql)) => {
            // This is fine, but we may still need to align the inner types.
            let new_inner = align_output_type(inner.as_ref(), inner_gql.as_ref());
            ty.clone().replace_inner(new_inner)
        }

        (RustType::List { inner, .. }, crate::schema::types::TypeRef::List(inner_gql)) => {
            // This is fine, but we may still need to align the inner types.
            let new_inner = align_output_type(inner.as_ref(), inner_gql.as_ref());
            ty.clone().replace_inner(new_inner)
        }
        (RustType::List { .. }, _) => {
            // Something weird is up if we hit this path so don't mess with anything.
            ty.clone()
        }
        (RustType::Box { inner, .. }, _) => {
            // This is fine, but we may still need to align the inner types.
            let new_inner = align_output_type(inner.as_ref(), gql_ty);
            ty.clone().replace_inner(new_inner)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use proc_macro2::TokenStream;
    use quote::quote;
    use rstest::rstest;
    use syn::parse2;

    use crate::{
        schema::{types::TypeRef, TypeIndex},
        types::parsing2::parse_rust_type,
    };

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
        #[case] graphql_type: TypeRef<'_, OutputType>,
        #[case] aligned_type: TokenStream,
    ) {
        let rust_type = parse2(rust_type).unwrap();
        let expected = parse2::<syn::Type>(aligned_type).unwrap();

        let rust_type = parse_rust_type(&rust_type);

        let result = align_output_type(&rust_type, &graphql_type).to_syn();

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
        #[case] graphql_type: TypeRef<'_, OutputType>,
    ) {
        let input = parse2(rust_type).unwrap();
        let rust_type = parse_rust_type(&input);

        let result = align_output_type(&rust_type, &graphql_type).to_syn();

        let input_quote = quote! { #input };
        let result_quote = quote! { #result };

        assert_eq!(input, result, "Expected {input_quote} got {result_quote}");
    }

    fn integer<'a, Kind>() -> TypeRef<'a, Kind> {
        TypeRef::Named("Int", TypeIndex::empty(), PhantomData)
    }

    fn list<Kind>(inner: TypeRef<'_, Kind>) -> TypeRef<'_, Kind> {
        TypeRef::List(Box::new(inner))
    }

    fn nullable<Kind>(inner: TypeRef<'_, Kind>) -> TypeRef<'_, Kind> {
        TypeRef::Nullable(Box::new(inner))
    }
}
