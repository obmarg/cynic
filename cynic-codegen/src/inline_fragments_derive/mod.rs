use darling::util::SpannedValue;
use proc_macro2::{Span, TokenStream};

use crate::{idents::PathExt, load_schema, schema, Errors, Ident};

pub mod input;

mod inline_fragments_impl;

#[cfg(test)]
mod tests;

pub use input::InlineFragmentsDeriveInput;

use crate::suggestions::{format_guess, guess_field};
use input::InlineFragmentsDeriveVariant;
use std::collections::HashSet;

pub fn inline_fragments_derive(ast: &syn::DeriveInput) -> Result<TokenStream, Errors> {
    use darling::FromDeriveInput;

    match InlineFragmentsDeriveInput::from_derive_input(ast) {
        Ok(input) => inline_fragments_derive_impl(input),
        Err(e) => Ok(e.write_errors()),
    }
}

pub(crate) fn inline_fragments_derive_impl(
    input: InlineFragmentsDeriveInput,
) -> Result<TokenStream, Errors> {
    use quote::{quote, quote_spanned};

    let schema =
        load_schema(&*input.schema_path).map_err(|e| e.into_syn_error(input.schema_path.span()))?;

    let target_type = find_union_or_interface_type(&input.graphql_type_name(), &schema);
    if target_type.is_none() {
        use graphql_parser::schema::{Definition, TypeDefinition};
        let candidates = schema.definitions.iter().flat_map(|def| match def {
            Definition::TypeDefinition(TypeDefinition::Union(union)) => Some(union.name.as_str()),
            Definition::TypeDefinition(TypeDefinition::Interface(interface)) => {
                Some(interface.name.as_str())
            }
            _ => None,
        });
        let guess_field = guess_field(candidates, &(input.graphql_type_name()));
        return Err(syn::Error::new(
            input.graphql_type_span(),
            format!(
                "Could not find a Union type or Interface named {}.{}",
                &input.graphql_type_name(),
                format_guess(guess_field)
            ),
        )
        .into());
    }
    let target_type = target_type.unwrap();

    let input_argument_struct = (&input.argument_struct).clone();
    let argument_struct = if let Some(arg_struct) = input_argument_struct {
        let span = arg_struct.span();
        let arg_struct_val: Ident = arg_struct.into();
        let argument_struct = quote_spanned! { span => #arg_struct_val };
        syn::parse2(argument_struct)?
    } else {
        syn::parse2(quote! { () })?
    };

    if let darling::ast::Data::Enum(variants) = &input.data {
        exhaustiveness_check(variants, &target_type, &schema)?;

        let fallback = check_fallback(variants, &target_type)?;

        let mut type_lock = input.schema_module();
        type_lock.push(Ident::for_type(input.graphql_type_name()));

        let fragments = fragments_from_variants(variants)?;

        let query_fragment_impl = QueryFragmentImpl {
            target_enum: input.ident.clone(),
            type_lock,
            argument_struct,
            fragments: &fragments,
            graphql_type_name: input.graphql_type_name(),
            fallback: fallback.clone(),
        };

        let inline_fragments_impl = inline_fragments_impl::InlineFragmentsImpl {
            target_enum: input.ident.clone(),
            fragments: &fragments,
            fallback,
        };

        Ok(quote! {
            #inline_fragments_impl
            #query_fragment_impl
        })
    } else {
        Err(syn::Error::new(
            Span::call_site(),
            "InlineFragments can only be derived from an enum".to_string(),
        )
        .into())
    }
}

fn exhaustiveness_check(
    variants: &[SpannedValue<InlineFragmentsDeriveVariant>],
    target_type: &InlineFragmentType,
    schema: &schema::Document,
) -> Result<(), Errors> {
    use schema::{Definition, TypeDefinition};

    let variant_names = variants
        .iter()
        .filter(|v| !*v.fallback)
        .map(|v| Ident::for_type(v.graphql_name()))
        .collect::<HashSet<_>>();

    let required_variants = match target_type {
        InlineFragmentType::Interface(iface) => schema
            .definitions
            .iter()
            .map(|d| match d {
                Definition::TypeDefinition(TypeDefinition::Object(obj)) => {
                    if obj.implements_interfaces.contains(&iface.name) {
                        Some(Ident::for_type(&obj.name))
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .flatten()
            .collect::<HashSet<_>>(),
        InlineFragmentType::Union(union) => union
            .types
            .iter()
            .map(Ident::for_type)
            .collect::<HashSet<_>>(),
    };

    let has_fallback = variants.iter().any(|v| *v.fallback);

    if has_fallback && !variant_names.is_subset(&required_variants) {
        let mut errors = Errors::default();

        for unexpected_variant_name in variant_names.difference(&required_variants) {
            let variant = variants
                .iter()
                .find(|v| Ident::for_type(v.graphql_name()) == *unexpected_variant_name)
                .unwrap();

            let candidates = required_variants.iter().map(|v| v.graphql_name());
            let guess_field = guess_field(candidates, &variant.graphql_name());
            errors.push(syn::Error::new(
                variant.span(),
                format!(
                    "Could not find a match for {} in {}.{}",
                    variant.graphql_name(),
                    target_type.name(),
                    format_guess(guess_field)
                ),
            ))
        }

        return Err(errors);
    } else if !has_fallback && variant_names != required_variants {
        let mut errors = Errors::default();

        for unexpected_variant_name in variant_names.difference(&required_variants) {
            let variant = variants
                .iter()
                .find(|v| Ident::for_type(v.graphql_name()) == *unexpected_variant_name)
                .unwrap();
            let candidates = required_variants.iter().map(|v| v.graphql_name());
            let guess_field = guess_field(candidates, &variant.graphql_name());
            errors.push(syn::Error::new(
                variant.span(),
                format!(
                    "Could not find a match for {} in {}.{}",
                    variant.graphql_name(),
                    target_type.name(),
                    format_guess(guess_field)
                ),
            ));
        }

        for missing_variant_name in required_variants.difference(&variant_names) {
            errors.push(syn::Error::new(
                Span::call_site(),
                format!(
                    "This InlineFragment is missing a variant for {}.  Either provide a variant for this type or add a fallback variant.",
                    missing_variant_name.graphql_name()
                ),
            ));
        }

        return Err(errors);
    }

    Ok(())
}

struct Fragment {
    rust_variant_name: syn::Ident,
    inner_type: syn::Type,
    graphql_type: String,
}

fn fragments_from_variants(
    variants: &[SpannedValue<InlineFragmentsDeriveVariant>],
) -> Result<Vec<Fragment>, syn::Error> {
    let mut result = vec![];
    for variant in variants {
        if *variant.fallback {
            continue;
        }

        if variant.fields.style != darling::ast::Style::Tuple || variant.fields.fields.len() != 1 {
            return Err(syn::Error::new(
                variant.span(),
                "InlineFragments derive requires enum variants to have one unnamed field",
            ));
        }
        let field = variant.fields.fields.first().unwrap();
        result.push(Fragment {
            rust_variant_name: variant.ident.clone(),
            inner_type: field.ty.clone(),
            graphql_type: variant.graphql_name(),
        });
    }
    Ok(result)
}

fn check_fallback(
    variants: &[SpannedValue<InlineFragmentsDeriveVariant>],
    target_type: &InlineFragmentType,
) -> Result<Option<(syn::Ident, Option<syn::Type>)>, Errors> {
    let fallbacks = variants.iter().filter(|v| *v.fallback).collect::<Vec<_>>();

    if fallbacks.is_empty() {
        return Ok(None);
    }

    if fallbacks.len() > 1 {
        let mut errors = Errors::default();
        for fallback in &fallbacks[1..] {
            errors.push(syn::Error::new(
                fallback.span(),
                "InlineFragments can't have more than one fallback",
            ))
        }

        return Err(errors);
    }

    let fallback = fallbacks[0];
    match target_type {
        InlineFragmentType::Interface(_) => match fallback.fields.style {
            darling::ast::Style::Struct => Err(syn::Error::new(
                fallback.span(),
                "InlineFragment fallbacks don't currently support struct variants",
            )
            .into()),
            darling::ast::Style::Tuple => {
                if fallback.fields.len() != 1 {
                    return Err(syn::Error::new(
                        fallback.span(),
                        "InlineFragments require variants to have one unnamed field",
                    )
                    .into());
                }
                Ok(Some((
                    fallback.ident.clone(),
                    Some(fallback.fields.fields[0].ty.clone()),
                )))
            }
            darling::ast::Style::Unit => Ok(Some((fallback.ident.clone(), None))),
        },
        InlineFragmentType::Union(_) => {
            if fallback.fields.style != darling::ast::Style::Unit {
                return Err(syn::Error::new(
                    fallback.span(),
                    "The fallback for a union type must be a unit variant",
                )
                .into());
            }
            Ok(Some((fallback.ident.clone(), None)))
        }
    }
}

struct QueryFragmentImpl<'a> {
    target_enum: syn::Ident,
    type_lock: syn::Path,
    argument_struct: syn::Type,
    fragments: &'a [Fragment],
    graphql_type_name: String,
    fallback: Option<(syn::Ident, Option<syn::Type>)>,
}

impl quote::ToTokens for QueryFragmentImpl<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let target_struct = &self.target_enum;
        let type_lock = &self.type_lock;
        let arguments = &self.argument_struct;
        let inner_types: Vec<_> = self
            .fragments
            .iter()
            .map(|fragment| &fragment.inner_type)
            .collect();
        let variants: Vec<_> = self
            .fragments
            .iter()
            .map(|fragment| &fragment.rust_variant_name)
            .collect();
        let graphql_type = proc_macro2::Literal::string(&self.graphql_type_name);

        tokens.append_all(quote! {
            #[automatically_derived]
            impl<'de> ::cynic::core::QueryFragment<'de> for #target_struct {
                type SchemaType = #type_lock;
                type Variables = #arguments;

                fn query(mut builder: ::cynic::queries::QueryBuilder<Self::SchemaType, Self::Variables>) {
                    #(
                        let fragment_builder = builder.inline_fragment();
                        let mut fragment_builder = fragment_builder.on::<<#inner_types as ::cynic::core::QueryFragment>::SchemaType>();
                        <#inner_types as ::cynic::core::QueryFragment>::query(
                            fragment_builder.select_children()
                        );
                    )*
                }
            }
        })
    }
}

fn find_union_or_interface_type<'a>(
    name: &str,
    schema: &'a schema::Document,
) -> Option<InlineFragmentType<'a>> {
    for definition in &schema.definitions {
        use graphql_parser::schema::{Definition, TypeDefinition};
        match definition {
            Definition::TypeDefinition(TypeDefinition::Union(union)) => {
                if union.name == name {
                    return Some(InlineFragmentType::Union(union));
                }
            }
            Definition::TypeDefinition(TypeDefinition::Interface(interface)) => {
                if interface.name == name {
                    return Some(InlineFragmentType::Interface(interface));
                }
            }
            _ => {}
        }
    }

    None
}

enum InlineFragmentType<'a> {
    Union(&'a schema::UnionType),
    Interface(&'a schema::InterfaceType),
}

impl<'a> InlineFragmentType<'a> {
    pub fn name(&self) -> &str {
        match self {
            InlineFragmentType::Interface(iface) => &iface.name,
            InlineFragmentType::Union(union) => &union.name,
        }
    }
}
