use proc_macro2::Span;
use rstest::rstest;
use syn::parse_quote;

use crate::schema::{parse_schema, types::Type, Schema};

use super::{analyse::analyse, parsing::CynicArguments};

#[rstest]
#[case::scalars("someScalarParams", parse_quote! { anInt: 1, aFloat: 3, anId: "hello" })]
#[case::missing_nullable_scalars("someNullableScalarParams", parse_quote! { anInt: 1 })]
#[case::empty_nullable_args("someNullableScalarParams", parse_quote! {})]
#[case::a_list_of_strings("filteredBooks", parse_quote! { filters: { authors: ["Charles Stross", "Ann Leckie"] } })]
#[case::list_wrapping("filteredBooks", parse_quote! { filters: { authors: "Ann Leckie" } })]
#[case::an_enum("filteredBooks", parse_quote! { filters: { state: "PUBLISHED" } })]
#[case::variable_in_object("filteredBooks", parse_quote! { filters: { state: $aVariable } })]
#[case::top_level_variable("filteredBooks", parse_quote! { filters: $aVaraible })]
#[case::top_level_variable("filteredBooks", parse_quote! { filters: $aVaraible, optionalFilters: $anotherVar })]
#[case::boolean_scalar("someNullableScalarParams", parse_quote! { aBool: true })]
#[case::missing_parameter("filteredBooks", parse_quote! {})]
#[case::unknown_parameter("someNullableScalarParams", parse_quote! { unknown: "hello" })]
fn test_analyse(#[case] field: &str, #[case] literals: CynicArguments) {
    use quote::format_ident;

    let schema_doc = parse_schema(SCHEMA).unwrap();
    let schema = Schema::new(&schema_doc).validate().unwrap();
    let ty = schema.lookup::<Type>("Query").ok().unwrap();
    let field = &ty.object().unwrap().field(field).unwrap();

    let literals = literals.arguments.into_iter().collect::<Vec<_>>();

    insta::assert_debug_snapshot!(analyse(
        literals,
        field,
        Some(&format_ident!("MyArguments").into()),
        Span::call_site()
    )
    .map(|o| o.arguments))
}

#[test]
fn test_analyse_errors_without_argument_struct() {
    let schema_doc = parse_schema(SCHEMA).unwrap();
    let schema = Schema::new(&schema_doc).validate().unwrap();
    let ty = schema.lookup::<Type>("Query").ok().unwrap();
    let field = &ty.object().unwrap().field("filteredBooks").unwrap();

    let literals: CynicArguments =
        parse_quote! { filters: $aVaraible, optionalFilters: $anotherVar };
    let literals = literals.arguments.into_iter().collect::<Vec<_>>();

    insta::assert_debug_snapshot!(
        analyse(literals, field, None, Span::call_site()).map(|o| o.arguments)
    )
}

const SCHEMA: &str = r#"
input BookFilters {
  authors: [String!]
  state: BookState
}

enum BookState {
  PUBLISHED
  OUT_OF_PRINT
}

type Query {
  filteredBooks(filters: BookFilters!, optionalFilters: BookFilters): [String!]!
  someScalarParams(anInt: Int!, aFloat: Float!, anId: ID!): [String]
  someNullableScalarParams(anInt: Int, aFloat: Float, anId: ID, aBool: Boolean): [String]
}
"#;
