use {
    darling::util::SpannedValue,
    proc_macro2::{Span, TokenStream},
};

use crate::{
    inline_fragments_derive::input::ValidationMode,
    load_schema,
    schema::{
        markers::TypeMarkerIdent,
        types::{InterfaceType, Kind, Type, UnionType},
        Schema, SchemaError,
    },
    variables_fields_path, Errors,
};

pub mod input;

mod inline_fragments_impl;

#[cfg(test)]
mod tests;

pub use input::InlineFragmentsDeriveInput;

use input::InlineFragmentsDeriveVariant;

use self::inline_fragments_impl::Fallback;

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

    let schema = Schema::new(&schema);

    let target_type = schema.lookup::<InlineFragmentType<'_>>(&input.graphql_type_name())?;

    input.validate(match target_type {
        InlineFragmentType::Union(_) => ValidationMode::Union,
        InlineFragmentType::Interface(_) => ValidationMode::Interface,
    })?;

    let variables = input.variables();

    if let darling::ast::Data::Enum(variants) = &input.data {
        let fallback = check_fallback(variants, &target_type)?;

        let type_lock = target_type.marker_ident().to_path(&input.schema_module());

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
    target_type: &InlineFragmentType<'_>,
) -> Result<Option<Fallback>, Errors> {
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

    match fallback.fields.style {
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
            Ok(Some(match target_type {
                InlineFragmentType::Interface(_) => Fallback::InterfaceVariant(
                    fallback.ident.clone(),
                    fallback.fields.fields[0].ty.clone(),
                ),
                InlineFragmentType::Union(_) => Fallback::UnionVariantWithTypename(
                    fallback.ident.clone(),
                    fallback.fields.fields[0].ty.clone(),
                ),
            }))
        }
        darling::ast::Style::Unit => Ok(Some(Fallback::UnionUnitVariant(fallback.ident.clone()))),
    }
}

struct QueryFragmentImpl<'a> {
    target_enum: syn::Ident,
    type_lock: syn::Path,
    variables: Option<syn::Path>,
    fragments: &'a [Fragment],
    graphql_type_name: String,
    fallback: Option<Fallback>,
}

impl quote::ToTokens for QueryFragmentImpl<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let target_struct = &self.target_enum;
        let type_lock = &self.type_lock;
        let variables_fields = variables_fields_path(self.variables.as_ref());
        let variables_fields = match &variables_fields {
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
            Some(Fallback::InterfaceVariant(_, fallback_fragment)) => quote! {
                <#fallback_fragment>::query(builder);
            },
            _ => quote! {},
        };

        tokens.append_all(quote! {
            #[automatically_derived]
            impl ::cynic::QueryFragment for #target_struct {
                type SchemaType = #type_lock;
                type VariablesFields = #variables_fields;

                const TYPE: Option<&'static str> = Some(#graphql_type);

                fn query(mut builder: ::cynic::queries::SelectionBuilder<'_, Self::SchemaType, Self::VariablesFields>) {
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

enum InlineFragmentType<'a> {
    Union(UnionType<'a>),
    Interface(InterfaceType<'a>),
}

impl<'a> InlineFragmentType<'a> {
    pub fn marker_ident(&self) -> TypeMarkerIdent<'a> {
        match self {
            InlineFragmentType::Union(inner) => inner.marker_ident(),
            InlineFragmentType::Interface(inner) => inner.marker_ident(),
        }
    }
}

impl<'a> TryFrom<Type<'a>> for InlineFragmentType<'a> {
    type Error = SchemaError;

    fn try_from(value: Type<'a>) -> Result<Self, Self::Error> {
        match value {
            Type::Interface(inner) => Ok(InlineFragmentType::Interface(inner)),
            Type::Union(inner) => Ok(InlineFragmentType::Union(inner)),
            _ => Err(SchemaError::unexpected_kind(value, Kind::UnionOrInterface)),
        }
    }
}
