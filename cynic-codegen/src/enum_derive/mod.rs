use proc_macro2::{Span, TokenStream};
use std::collections::BTreeMap;

use crate::{
    idents::RenameAll,
    load_schema,
    schema::{
        types::{EnumType, EnumValue},
        Schema, Unvalidated,
    },
};

pub(crate) mod input;
use crate::suggestions::{format_guess, guess_field};
pub use input::EnumDeriveInput;
use input::EnumDeriveVariant;

pub fn enum_derive(ast: &syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    use darling::FromDeriveInput;
    use syn::spanned::Spanned;

    let enum_span = ast.span();

    match EnumDeriveInput::from_derive_input(ast) {
        Ok(input) => {
            let schema_doc = load_schema(&*input.schema_path)
                .map_err(|e| e.into_syn_error(input.schema_path.span()))?;

            let schema = Schema::new(&schema_doc);

            enum_derive_impl(input, &schema, enum_span).or_else(|e| Ok(e.to_compile_error()))
        }
        Err(e) => Ok(e.write_errors()),
    }
}

pub fn enum_derive_impl(
    input: EnumDeriveInput,
    schema: &Schema<'_, Unvalidated>,
    enum_span: Span,
) -> Result<TokenStream, syn::Error> {
    use quote::quote;

    let enum_def = schema
        .lookup::<EnumType>(&input.graphql_type_name())
        .map_err(|e| syn::Error::new(input.graphql_type_span(), e))?;

    let rename_all = input.rename_all.unwrap_or(RenameAll::ScreamingSnakeCase);

    if let darling::ast::Data::Enum(variants) = &input.data {
        let pairs = match join_variants(
            variants,
            &enum_def,
            &input.ident.to_string(),
            rename_all,
            &enum_span,
        ) {
            Ok(pairs) => pairs,
            Err(error_tokens) => return Ok(error_tokens),
        };

        let graphql_type_name = proc_macro2::Literal::string(&input.graphql_type_name());
        let enum_marker_ident = proc_macro2::Ident::from(enum_def.marker_ident());

        let string_literals: Vec<_> = pairs
            .iter()
            .map(|(_, value)| value.name.to_literal())
            .collect();

        let variants: Vec<_> = pairs.iter().map(|(variant, _)| &variant.ident).collect();
        let variant_indexes: Vec<_> = pairs
            .iter()
            .enumerate()
            .map(|(i, _)| {
                proc_macro2::Literal::u32_suffixed(
                    i.try_into().expect("an enum with less than 2^32 variants"),
                )
            })
            .collect();

        let schema_module = input.schema_module();
        let ident = input.ident;

        Ok(quote! {
            #[automatically_derived]
            impl ::cynic::Enum for #ident {
                type SchemaType = #schema_module::#enum_marker_ident;
            }

            #[automatically_derived]
            impl ::cynic::serde::Serialize for #ident {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: ::cynic::serde::Serializer {
                        match self {
                            #(
                                #ident::#variants => serializer.serialize_unit_variant(#graphql_type_name, #variant_indexes, #string_literals),
                            )*
                        }
                    }
            }

            #[automatically_derived]
            impl<'de> ::cynic::serde::Deserialize<'de> for #ident {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: ::cynic::serde::Deserializer<'de>,
                {
                    match <String as ::cynic::serde::Deserialize>::deserialize(deserializer)?.as_ref() {
                        #(
                            #string_literals => Ok(#ident::#variants),
                        )*
                        unknown => {
                            const VARIANTS: &'static [&'static str] = &[#(#string_literals),*];
                            Err(::cynic::serde::de::Error::unknown_variant(unknown, VARIANTS))
                        }
                    }
                }
            }

            ::cynic::impl_coercions!(#ident, #schema_module::#enum_marker_ident);

            #[automatically_derived]
            impl #schema_module::variable::Variable for #ident {
                const TYPE: ::cynic::variables::VariableType = ::cynic::variables::VariableType::Named(#graphql_type_name);
            }
        })
    } else {
        Err(syn::Error::new(
            enum_span,
            "Enum can only be derived from an enum".to_string(),
        ))
    }
}

fn join_variants<'a>(
    variants: &'a [EnumDeriveVariant],
    enum_def: &'a EnumType<'a>,
    enum_name: &str,
    rename_all: RenameAll,
    enum_span: &Span,
) -> Result<Vec<(&'a EnumDeriveVariant, &'a EnumValue<'a>)>, TokenStream> {
    let mut map = BTreeMap::new();
    for variant in variants {
        let graphql_ident = variant.graphql_ident(rename_all);
        map.insert(
            graphql_ident.graphql_name(),
            (Some(variant), enum_def.value(&graphql_ident)),
        );
    }

    for value in &enum_def.values {
        if !map.contains_key(value.name.as_str()) {
            map.insert(value.name.as_str().to_owned(), (None, Some(value)));
        }
    }

    let mut missing_variants = vec![];
    let mut errors = TokenStream::new();
    for (graphql_name, value) in map.iter() {
        match value {
            (None, Some(enum_value)) => missing_variants.push(enum_value.name.as_str()),
            (Some(variant), None) => {
                let candidates = map
                    .values()
                    .flat_map(|v| v.1.map(|input| input.name.as_str()));
                let guess_field = guess_field(candidates, &(*(graphql_name)));
                errors.extend(
                    syn::Error::new(
                        variant.ident.span(),
                        format!(
                            "Could not find a variant {} in the GraphQL enum {}.{}",
                            graphql_name,
                            enum_name,
                            format_guess(guess_field)
                        ),
                    )
                    .to_compile_error(),
                )
            }
            _ => (),
        }
    }
    if !missing_variants.is_empty() {
        let missing_variants_string = missing_variants.join(", ");
        errors.extend(
            syn::Error::new(
                *enum_span,
                format!("Missing variants: {}", missing_variants_string),
            )
            .to_compile_error(),
        )
    }
    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(map
        .into_iter()
        .map(|(_, (a, b))| (a.unwrap(), b.unwrap()))
        .collect())
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use darling::util::SpannedValue;
    use rstest::rstest;
    use std::collections::HashSet;
    use syn::parse_quote;

    use super::*;
    use crate::schema::FieldName;

    #[rstest(
        enum_variant_1,
        enum_variant_2,
        enum_value_1,
        enum_value_2,
        rename_rule,
        case(
            "Cheesecake",
            "IceCream",
            "CHEESECAKE",
            "ICE_CREAM",
            RenameAll::ScreamingSnakeCase
        ),
        case("CHEESECAKE", "ICE_CREAM", "CHEESECAKE", "ICE_CREAM", RenameAll::None)
    )]
    fn join_variants_happy_path(
        enum_variant_1: &str,
        enum_variant_2: &str,
        enum_value_1: &str,
        enum_value_2: &str,
        rename_rule: RenameAll,
    ) {
        let variants = vec![
            EnumDeriveVariant {
                ident: proc_macro2::Ident::new(enum_variant_1, Span::call_site()),
                rename: None,
            },
            EnumDeriveVariant {
                ident: proc_macro2::Ident::new(enum_variant_2, Span::call_site()),
                rename: None,
            },
        ];
        let mut gql_enum = EnumType {
            description: None,
            name: "Desserts",
            values: vec![],
        };
        gql_enum.values.push(EnumValue {
            name: FieldName::new(enum_value_1),
            description: None,
        });
        gql_enum.values.push(EnumValue {
            name: FieldName::new(enum_value_2),
            description: None,
        });

        let result = join_variants(
            &variants,
            &gql_enum,
            "Desserts",
            rename_rule,
            &Span::call_site(),
        );

        assert_matches!(result, Ok(_));
        let pairs = result.unwrap();

        assert_eq!(pairs.len(), 2);

        let names: HashSet<_> = pairs
            .iter()
            .map(|(variant, ty)| (variant.ident.to_string(), ty.name.clone()))
            .collect();

        assert_eq!(
            names,
            maplit::hashset! {(enum_variant_1.into(), FieldName::new(enum_value_1)), (enum_variant_2.into(), FieldName::new(enum_value_2))}
        );
    }

    #[test]
    fn join_variants_with_field_rename() {
        let variants = vec![
            EnumDeriveVariant {
                ident: proc_macro2::Ident::new("Cheesecake", Span::call_site()),
                rename: None,
            },
            EnumDeriveVariant {
                ident: proc_macro2::Ident::new("IceCream", Span::call_site()),
                rename: Some(SpannedValue::new("iced-goodness".into(), Span::call_site())),
            },
        ];
        let mut gql_enum = EnumType {
            description: None,
            name: "Desserts",
            values: vec![],
        };
        gql_enum.values.push(EnumValue {
            name: FieldName::new("CHEESECAKE"),
            description: None,
        });
        gql_enum.values.push(EnumValue {
            name: FieldName::new("iced-goodness"),
            description: None,
        });

        let result = join_variants(
            &variants,
            &gql_enum,
            "Desserts",
            RenameAll::ScreamingSnakeCase,
            &Span::call_site(),
        );

        assert_matches!(result, Ok(_));
        let pairs = result.unwrap();

        assert_eq!(pairs.len(), 2);

        let names: HashSet<_> = pairs
            .iter()
            .map(|(variant, ty)| (variant.ident.to_string(), ty.name.clone()))
            .collect();

        assert_eq!(
            names,
            maplit::hashset! {("Cheesecake".into(), FieldName::new("CHEESECAKE")), ("IceCream".into(), FieldName::new("iced-goodness"))}
        );
    }

    #[test]
    fn join_variants_missing_rust_variant() {
        let variants = vec![EnumDeriveVariant {
            ident: proc_macro2::Ident::new("CHEESECAKE", Span::call_site()),
            rename: None,
        }];
        let mut gql_enum = EnumType {
            description: None,
            name: "Desserts",
            values: vec![],
        };
        gql_enum.values.push(EnumValue {
            name: FieldName::new("CHEESECAKE"),
            description: None,
        });
        gql_enum.values.push(EnumValue {
            name: FieldName::new("ICE_CREAM"),
            description: None,
        });

        let result = join_variants(
            &variants,
            &gql_enum,
            "Desserts",
            RenameAll::None,
            &Span::call_site(),
        );

        assert_matches!(result, Err(_));
    }

    #[test]
    fn join_variants_missing_gql_variant() {
        let variants = vec![EnumDeriveVariant {
            ident: proc_macro2::Ident::new("CHEESECAKE", Span::call_site()),
            rename: None,
        }];
        let mut gql_enum = EnumType {
            description: None,
            name: "Desserts",
            values: vec![],
        };
        gql_enum.values.push(EnumValue {
            name: FieldName::new("ICE_CREAM"),
            description: None,
        });

        let result = join_variants(
            &variants,
            &gql_enum,
            "Desserts",
            RenameAll::None,
            &Span::call_site(),
        );

        assert_matches!(result, Err(_));
    }

    #[rstest(input => [
        parse_quote!(
            #[cynic(
                schema_path = "../schemas/test_cases.graphql",
            )]
            enum States {
                Open,
                Closed,
                Deleted
            }
        ),
    ])]
    fn snapshot_enum_derive(input: syn::DeriveInput) {
        let tokens = enum_derive(&input).unwrap();

        insta::assert_snapshot!(format_code(format!("{}", tokens)));
    }

    fn format_code(input: String) -> String {
        use std::io::Write;

        let mut cmd = std::process::Command::new("rustfmt")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit())
            .spawn()
            .expect("failed to execute rustfmt");

        write!(cmd.stdin.as_mut().unwrap(), "{}", input).unwrap();

        std::str::from_utf8(&cmd.wait_with_output().unwrap().stdout)
            .unwrap()
            .to_owned()
    }
}
