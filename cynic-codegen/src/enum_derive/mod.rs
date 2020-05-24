use darling::util::SpannedValue;
use graphql_parser::schema::{Definition, Document, EnumType, EnumValue, TypeDefinition};
use proc_macro2::{Span, TokenStream};
use std::collections::{HashMap, HashSet};

use crate::{load_schema, Ident};

mod input;

pub use input::EnumDeriveInput;
use input::EnumDeriveVariant;

pub fn enum_derive(ast: &syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    use darling::FromDeriveInput;

    match EnumDeriveInput::from_derive_input(ast) {
        Ok(input) => load_schema(&*input.schema_path)
            .map_err(|e| e.to_syn_error(input.schema_path.span()))
            .and_then(|schema| enum_derive_impl(input, &schema))
            .or_else(|e| Ok(e.to_compile_error())),
        Err(e) => Ok(e.write_errors()),
    }
}

pub fn enum_derive_impl(
    input: EnumDeriveInput,
    schema: &Document,
) -> Result<TokenStream, syn::Error> {
    use quote::{quote, quote_spanned};

    let enum_def = schema.definitions.iter().find_map(|def| {
        if let Definition::TypeDefinition(TypeDefinition::Enum(e)) = def {
            if e.name == *input.graphql_type {
                return Some(e);
            }
        }
        None
    });
    if enum_def.is_none() {
        return Err(syn::Error::new(
            input.graphql_type.span(),
            format!(
                "Could not find an enum named {} in {}",
                *input.graphql_type, *input.schema_path
            ),
        ));
    }
    let enum_def = enum_def.unwrap();

    if let darling::ast::Data::Enum(variants) = &input.data {
        let pairs = match join_variants(variants, enum_def, &input.ident.to_string()) {
            Ok(pairs) => pairs,
            Err(error_tokens) => return Ok(error_tokens),
        };

        let ident = input.ident;

        let string_literals: Vec<_> = pairs
            .iter()
            .map(|(_, value)| proc_macro2::Literal::string(&value.name))
            .collect();

        let variants: Vec<_> = pairs.iter().map(|(variant, _)| &variant.ident).collect();

        Ok(quote! {
            impl ::cynic::Enum for #ident {
                fn select() -> cynic::SelectionSet<'static, Self, ()> {
                    ::cynic::selection_set::string().and_then(|s| {
                        match s.as_ref() {
                            #(
                                #string_literals => ::cynic::selection_set::succeed(#ident::#variants),
                            )*
                            _ => ::cynic::selection_set::fail(format!("Unknown variant: {}", &s))
                        }
                    })
                }
            }
        })
    } else {
        Err(syn::Error::new(
            Span::call_site(),
            format!("Enum can only be derived from an enum"),
        ))
    }
}

fn join_variants<'a>(
    variants: &'a [EnumDeriveVariant],
    enum_def: &'a EnumType,
    enum_name: &str,
) -> Result<Vec<(&'a EnumDeriveVariant, &'a EnumValue)>, TokenStream> {
    let mut map = HashMap::new();
    for variant in variants {
        map.insert(variant.ident.clone().into(), (Some(variant), None));
    }

    for value in &enum_def.values {
        let our_ident = Ident::for_variant(&value.name);
        let mut entry = map.entry(our_ident).or_insert((None, None));
        entry.1 = Some(value);
    }

    let mut missing_variants = vec![];
    let mut errors = TokenStream::new();
    for (_, value) in map.iter() {
        match value {
            (None, Some(enum_value)) => missing_variants.push(enum_value.name.as_ref()),
            (Some(variant), None) => errors.extend(
                syn::Error::new(
                    variant.ident.span(),
                    format!(
                        "Could not find a variant {} in the GraphQL enum {}",
                        variant.ident.to_string(),
                        enum_name
                    ),
                )
                .to_compile_error(),
            ),
            _ => (),
        }
    }
    if !missing_variants.is_empty() {
        let missing_variants_string = missing_variants.join(", ");
        errors.extend(
            syn::Error::new(
                Span::call_site(),
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
    use super::*;
    use assert_matches::assert_matches;

    #[test]
    fn join_variants_happy_path() {
        let variants = vec![
            EnumDeriveVariant {
                ident: proc_macro2::Ident::new("Cheesecake", Span::call_site()),
            },
            EnumDeriveVariant {
                ident: proc_macro2::Ident::new("IceCream", Span::call_site()),
            },
        ];
        let mut gql_enum = EnumType::new("Desserts".into());
        gql_enum.values.push(EnumValue::new("CHEESECAKE".into()));
        gql_enum.values.push(EnumValue::new("ICE_CREAM".into()));

        let result = join_variants(&variants, &gql_enum, "Desserts");

        assert_matches!(result, Ok(_));
        let pairs = result.unwrap();

        assert_eq!(pairs.len(), 2);

        let names: HashSet<_> = pairs
            .iter()
            .map(|(variant, ty)| (variant.ident.to_string(), ty.name.clone()))
            .collect();

        assert_eq!(
            names,
            maplit::hashset! {("Cheesecake".into(), "CHEESECAKE".into()), ("IceCream".into(), "ICE_CREAM".into())}
        );
    }

    #[test]
    fn join_variants_missing_rust_variant() {
        let variants = vec![EnumDeriveVariant {
            ident: proc_macro2::Ident::new("Cheesecake", Span::call_site()),
        }];
        let mut gql_enum = EnumType::new("Desserts".into());
        gql_enum.values.push(EnumValue::new("CHEESECAKE".into()));
        gql_enum.values.push(EnumValue::new("ICE_CREAM".into()));

        let result = join_variants(&variants, &gql_enum, "Desserts");

        assert_matches!(result, Err(_));
    }

    #[test]
    fn join_variants_missing_gql_variant() {
        let variants = vec![EnumDeriveVariant {
            ident: proc_macro2::Ident::new("Cheesecake", Span::call_site()),
        }];
        let mut gql_enum = EnumType::new("Desserts".into());
        gql_enum.values.push(EnumValue::new("ICE_CREAM".into()));

        let result = join_variants(&variants, &gql_enum, "Desserts");

        assert_matches!(result, Err(_));
    }
}
