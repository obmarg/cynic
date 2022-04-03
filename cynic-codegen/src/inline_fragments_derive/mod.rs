use darling::util::SpannedValue;
use proc_macro2::{Span, TokenStream};

use crate::{
    idents::PathExt, inline_fragments_derive::input::ValidationMode, load_schema, schema, Errors,
    Ident,
};

pub mod input;

mod inline_fragments_impl;

#[cfg(test)]
mod tests;

pub use input::InlineFragmentsDeriveInput;

use crate::suggestions::{format_guess, guess_field};
use input::InlineFragmentsDeriveVariant;

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
    use quote::quote;

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

    input.validate(match target_type {
        InlineFragmentType::Union(_) => ValidationMode::Union,
        InlineFragmentType::Interface(_) => ValidationMode::Interface,
    })?;

    let variables = input.variables();

    if let darling::ast::Data::Enum(variants) = &input.data {
        let fallback = check_fallback(variants, &target_type)?;

        let mut type_lock = input.schema_module();
        type_lock.push(Ident::for_type(input.graphql_type_name()));

        let fragments = fragments_from_variants(variants)?;

        let query_fragment_impl = QueryFragmentImpl {
            target_enum: input.ident.clone(),
            type_lock,
            variables,
            fragments: &fragments,
            graphql_type_name: input.graphql_type_name(),
            fallback: fallback.clone(),
        };

        let inline_fragments_impl = inline_fragments_impl::InlineFragmentsImpl {
            target_enum: input.ident.clone(),
            fragments: &fragments,
            fallback,
        };

        let deprecations = input.deprecations();

        Ok(quote! {
            #inline_fragments_impl
            #query_fragment_impl
            #deprecations
        })
    } else {
        Err(syn::Error::new(
            Span::call_site(),
            "InlineFragments can only be derived from an enum".to_string(),
        )
        .into())
    }
}

struct Fragment {
    rust_variant_name: syn::Ident,
    inner_type: syn::Type,
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
    variables: Option<syn::Path>,
    fragments: &'a [Fragment],
    graphql_type_name: String,
    fallback: Option<(syn::Ident, Option<syn::Type>)>,
}

impl quote::ToTokens for QueryFragmentImpl<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let target_struct = &self.target_enum;
        let type_lock = &self.type_lock;
        let variables = match &self.variables {
            Some(path) => quote! { #path },
            None => quote! { () },
        };
        let inner_types: Vec<_> = self
            .fragments
            .iter()
            .map(|fragment| &fragment.inner_type)
            .collect();
        let graphql_type = proc_macro2::Literal::string(&self.graphql_type_name);
        let fallback_selection = match &self.fallback {
            Some((_, Some(fallback_fragment))) => quote! {
                <#fallback_fragment>::query(builder);
            },
            _ => quote! {},
        };

        tokens.append_all(quote! {
            #[automatically_derived]
            impl<'de> ::cynic::QueryFragment<'de> for #target_struct {
                type SchemaType = #type_lock;
                type Variables = #variables;

                const TYPE: Option<&'static str> = Some(#graphql_type);

                fn query(mut builder: ::cynic::queries::SelectionBuilder<Self::SchemaType, Self::Variables>) {
                    #(
                        let fragment_builder = builder.inline_fragment();
                        let mut fragment_builder = fragment_builder.on::<<#inner_types as ::cynic::QueryFragment>::SchemaType>();
                        <#inner_types as ::cynic::QueryFragment>::query(
                            fragment_builder.select_children()
                        );
                    )*

                    #fallback_selection
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
