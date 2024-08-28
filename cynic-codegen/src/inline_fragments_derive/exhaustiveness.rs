use std::collections::HashSet;

use darling::util::SpannedValue;
use proc_macro2::Span;

use crate::{
    error::Errors,
    suggestions::{format_guess, guess_field},
};

use super::{input::InlineFragmentsDeriveVariant, InlineFragmentType};

pub(super) fn exhaustiveness_check(
    variants: &[SpannedValue<InlineFragmentsDeriveVariant>],
    target_type: &InlineFragmentType<'_>,
) -> Result<(), Errors> {
    let variant_names = variants
        .iter()
        .filter(|v| !*v.fallback)
        .map(|v| v.ident.to_string())
        .collect::<HashSet<_>>();

    let InlineFragmentType::Union(union_type) = target_type else {
        // I can't be bothered implementing exhaustiveness for interfaces now
        // so I'm just making it unsupported. If anyone wants it they're welcome to
        // contribute an implementation
        unreachable!()
    };

    let required_variants = union_type
        .types
        .iter()
        .map(|ty| ty.to_string())
        .collect::<HashSet<_>>();

    if variant_names != required_variants {
        let mut errors = Errors::default();

        for unexpected_variant_name in variant_names.difference(&required_variants) {
            let variant = variants
                .iter()
                .find(|v| v.ident == *unexpected_variant_name)
                .unwrap();
            let guess_field = guess_field(
                required_variants
                    .iter()
                    .map(|variant_name| variant_name.as_str()),
                unexpected_variant_name,
            );
            errors.push(syn::Error::new(
                variant.span(),
                format!(
                    "Could not find a member named {} in the union {}. {}",
                    unexpected_variant_name,
                    target_type.name(),
                    format_guess(guess_field)
                ),
            ));
        }

        for missing_variant_name in required_variants.difference(&variant_names) {
            errors.push(syn::Error::new(
                Span::call_site(),
                format!(
                    "This InlineFragment is missing a variant for {}.  Either provide a variant for this type or remove the exhaustive attribute",
                    missing_variant_name
                ),
            ));
        }

        return Err(errors);
    }

    Ok(())
}
