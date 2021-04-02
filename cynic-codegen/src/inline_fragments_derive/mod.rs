use darling::util::SpannedValue;
use proc_macro2::{Span, TokenStream};

use crate::{load_schema, schema, Errors, Ident, TypePath};

pub mod input;

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

        let inline_fragments_impl = InlineFragmentsImpl {
            target_struct: input.ident.clone(),
            type_lock: TypePath::concat(&[
                Ident::new_spanned(&*input.query_module, input.query_module.span()).into(),
                Ident::for_type(&input.graphql_type_name()).into(),
            ]),
            argument_struct,
            possible_types: possible_types_from_variants(variants)?,
            graphql_type_name: input.graphql_type_name(),
            fallback,
        };

        Ok(quote! { #inline_fragments_impl })
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
        .map(|v| v.ident.to_string())
        .collect::<HashSet<_>>();

    let required_variants = match target_type {
        InlineFragmentType::Interface(iface) => schema
            .definitions
            .iter()
            .map(|d| match d {
                Definition::TypeDefinition(TypeDefinition::Object(obj)) => {
                    if obj.implements_interfaces.contains(&iface.name) {
                        Some(&obj.name)
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .flatten()
            .cloned()
            .collect::<HashSet<_>>(),
        InlineFragmentType::Union(union) => union.types.iter().cloned().collect::<HashSet<_>>(),
    };

    let has_fallback = variants.iter().any(|v| *v.fallback);

    if has_fallback && !variant_names.is_subset(&required_variants) {
        let mut errors = Errors::default();

        for unexpected_variant_name in variant_names.difference(&required_variants) {
            let variant = variants
                .iter()
                .find(|v| v.ident == *unexpected_variant_name)
                .unwrap();

            let candidates = required_variants.iter().map(|v| v.as_str());
            let guess_field = guess_field(candidates, variant.ident.to_string().as_str());
            errors.push(syn::Error::new(
                variant.span(),
                format!(
                    "Could not find a match for {} in {}.{}",
                    variant.ident,
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
                .find(|v| v.ident == *unexpected_variant_name)
                .unwrap();
            let candidates = required_variants.iter().map(|v| v.as_str());
            let guess_field = guess_field(candidates, variant.ident.to_string().as_str());
            errors.push(syn::Error::new(
                variant.span(),
                format!(
                    "Could not find a match for {} in {}.{}",
                    variant.ident,
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
                    missing_variant_name
                ),
            ));
        }

        return Err(errors);
    }

    Ok(())
}

fn possible_types_from_variants(
    variants: &[SpannedValue<InlineFragmentsDeriveVariant>],
) -> Result<Vec<(syn::Ident, syn::Type)>, syn::Error> {
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
        result.push((variant.ident.clone(), field.ty.clone()));
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

struct InlineFragmentsImpl {
    target_struct: syn::Ident,
    type_lock: TypePath,
    argument_struct: syn::Type,
    possible_types: Vec<(syn::Ident, syn::Type)>,
    graphql_type_name: String,
    fallback: Option<(syn::Ident, Option<syn::Type>)>,
}

impl quote::ToTokens for InlineFragmentsImpl {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let target_struct = &self.target_struct;
        let type_lock = &self.type_lock;
        let arguments = &self.argument_struct;
        let internal_types: Vec<_> = self.possible_types.iter().map(|(_, ty)| ty).collect();
        let variants: Vec<_> = self.possible_types.iter().map(|(v, _)| v).collect();
        let graphql_type = proc_macro2::Literal::string(&self.graphql_type_name);

        let fallback_selection = if let Some((fallback_variant, fallback_type)) = &self.fallback {
            if let Some(fallback_type) = fallback_type {
                quote! {
                    use ::cynic::QueryFragment;
                    Some(
                        #fallback_type
                            ::fragment(
                                context.with_args(
                                    ::cynic::FromArguments::from_arguments(context.args)
                                )
                            )
                            .map(#target_struct::#fallback_variant)
                            .transform_typelock()
                    )
                }
            } else {
                quote! {
                    Some(
                        ::cynic::selection_set::succeed_using(
                            || #target_struct::#fallback_variant
                        )
                    )
                }
            }
        } else {
            quote! { None }
        };

        tokens.append_all(quote! {
            #[automatically_derived]
            impl ::cynic::InlineFragments for #target_struct {
                type TypeLock = #type_lock;
                type Arguments = #arguments;

                fn fragments(context: ::cynic::FragmentContext<'_, Self::Arguments>) ->
                    Vec<(String, ::cynic::SelectionSet<'static, Self, Self::TypeLock>)>
                {
                    use ::cynic::QueryFragment;

                    let args = context.args;

                    let mut rv = vec![];
                    #(
                        rv.push((
                            #internal_types::graphql_type(),
                            #internal_types
                                ::fragment(context.with_args(::cynic::FromArguments::from_arguments(args)))
                                .map(#target_struct::#variants)
                                .transform_typelock()
                        ));
                    )*
                    rv
                }

                fn graphql_type() -> String {
                    #graphql_type.to_string()
                }

                fn fallback(context: ::cynic::FragmentContext<'_, Self::Arguments>) ->
                  Option<::cynic::SelectionSet<'static, Self, Self::TypeLock>>
                {
                    #fallback_selection
                }
            }
        });
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
