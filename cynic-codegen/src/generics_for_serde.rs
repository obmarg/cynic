use darling::usage::IdentRefSet;
use syn::parse_quote;

pub(crate) fn with_de_and_deserialize_bounds(generics: &syn::Generics) -> syn::Generics {
    let mut generics_with_de_and_deserialize_bounds = generics.clone();
    generics_with_de_and_deserialize_bounds
        .params
        .push(parse_quote!('de));
    for generic in &generics.params {
        match generic {
            syn::GenericParam::Type(type_) => {
                let ident = &type_.ident;
                generics_with_de_and_deserialize_bounds
                    .make_where_clause()
                    .predicates
                    .push(parse_quote! { #ident: cynic::serde::Deserialize<'de> })
            }
            syn::GenericParam::Lifetime(_) | syn::GenericParam::Const(_) => {}
        }
    }
    generics_with_de_and_deserialize_bounds
}

pub(crate) fn with_serialize_bounds(generics: &syn::Generics) -> syn::Generics {
    let mut generics_with_serialize_bounds = generics.clone();
    for generic in &generics.params {
        match generic {
            syn::GenericParam::Type(type_) => {
                let ident = &type_.ident;
                generics_with_serialize_bounds
                    .make_where_clause()
                    .predicates
                    .push(parse_quote! { #ident: cynic::serde::Serialize })
            }
            syn::GenericParam::Lifetime(_) | syn::GenericParam::Const(_) => {}
        }
    }
    generics_with_serialize_bounds
}

pub(crate) fn with_selective_serialize_bounds(
    generics: &syn::Generics,
    only_generics_named: IdentRefSet<'_>,
) -> syn::Generics {
    let mut generics_with_serialize_bounds = generics.clone();
    for generic in &generics.params {
        match generic {
            syn::GenericParam::Type(type_) if only_generics_named.contains(&type_.ident) => {
                let ident = &type_.ident;
                generics_with_serialize_bounds
                    .make_where_clause()
                    .predicates
                    .push(parse_quote! { #ident: cynic::serde::Serialize })
            }
            syn::GenericParam::Type(_)
            | syn::GenericParam::Lifetime(_)
            | syn::GenericParam::Const(_) => {}
        }
    }
    generics_with_serialize_bounds
}
