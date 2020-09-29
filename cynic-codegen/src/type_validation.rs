use crate::FieldType;

pub fn check_types_are_compatible(
    gql_type: &FieldType,
    rust_type: &syn::Type,
    flattening: bool,
) -> Result<(), syn::Error> {
    use quote::quote;
    use syn::spanned::Spanned;

    let parsed_type = parse_type(rust_type);

    if parsed_type == ParsedType::Unknown {
        return Err(syn::Error::new(
            rust_type.span(),
            "Cynic does not understand this type. Only un-parameterised types, Vecs & Options are accepted currently.",
        ));
    }

    // @TODO process `ParsedType::Box`?

    if gql_type.is_nullable() {
        if let ParsedType::Optional(inner) = parsed_type {
            return check_types_are_compatible(&gql_type.as_required(), &inner, flattening);
        } else if !flattening {
            // If we're flattening then it's all good.  But otherwise we should return an error.
            return Err(syn::Error::new(
                        rust_type.span(),
                        format!(
                            "This GraphQL type is optional but you're not wrapping the type in Option.  Did you mean Option<{}>",
                            quote! { #rust_type }
                        )
                    ));
        }
    } else if let ParsedType::Optional(inner) = parsed_type {
        return Err(syn::Error::new(
                        rust_type.span(),
                        format!(
                            "This GraphQL type is required but you're wrapping the type in Option.  Did you mean {}",
                            quote! { #inner }
                        )
                    ));
    } else if let FieldType::List(item_type, _) = gql_type {
        if let ParsedType::List(inner) = parsed_type {
            return check_types_are_compatible(&item_type, &inner, flattening);
        } else if !flattening {
            // If we're flattening then it's all good.  But otherwise we should return an error.
            return Err(syn::Error::new(
                        rust_type.span(),
                        format!(
                            "This GraphQL type is a list but you're not wrapping the type in Vec.  Did you mean Vec<{}>",
                            quote! { #rust_type }
                        )
                    ));
        }
    } else if let ParsedType::List(inner) = parsed_type {
        return Err(syn::Error::new(
                        rust_type.span(),
                        format!(
                            "This GraphQL type is not a list but you're wrapping the type in Vec.  Did you mean {}",
                            quote! { #inner }
                        )
                    ));
    }

    Ok(())
}

/// A simplified rust type structure
#[derive(Debug, PartialEq)]
enum ParsedType<'a> {
    Optional(&'a syn::Type),
    List(&'a syn::Type),
    Box(&'a syn::Type),
    SimpleType,
    Unknown,
}

fn parse_type<'a>(ty: &'a syn::Type) -> ParsedType<'a> {
    if let syn::Type::Path(type_path) = ty {
        if let Some(last_segment) = type_path.path.segments.last() {
            if last_segment.ident.to_string() == "Box" {
                if let syn::PathArguments::AngleBracketed(angle_bracketed) = &last_segment.arguments
                {
                    for arg in &angle_bracketed.args {
                        if let syn::GenericArgument::Type(inner_type) = arg {
                            return ParsedType::Box(inner_type);
                        }
                    }
                }
                return ParsedType::Unknown;
            }
            if last_segment.ident.to_string() == "Option" {
                if let syn::PathArguments::AngleBracketed(angle_bracketed) = &last_segment.arguments
                {
                    for arg in &angle_bracketed.args {
                        if let syn::GenericArgument::Type(inner_type) = arg {
                            return ParsedType::Optional(inner_type);
                        }
                    }
                }
                return ParsedType::Unknown;
            } else if last_segment.ident.to_string() == "Vec" {
                if let syn::PathArguments::AngleBracketed(angle_bracketed) = &last_segment.arguments
                {
                    for arg in &angle_bracketed.args {
                        if let syn::GenericArgument::Type(inner_type) = arg {
                            return ParsedType::List(inner_type);
                        }
                    }
                }
                return ParsedType::Unknown;
            }

            if last_segment.arguments == syn::PathArguments::None {
                return ParsedType::SimpleType;
            }
        }
    }

    ParsedType::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FieldType, Ident};
    use assert_matches::assert_matches;
    use quote::quote;

    #[test]
    fn test_required_validation() {
        let required_field = FieldType::Scalar(Ident::new("").into(), false);
        let optional_field = FieldType::Scalar(Ident::new("").into(), true);

        assert_matches!(
            check_types_are_compatible(
                &required_field,
                &syn::parse2(quote! { i32 }).unwrap(),
                false
            ),
            Ok(())
        );
        assert_matches!(
            check_types_are_compatible(
                &optional_field,
                &syn::parse2(quote! { Option<i32> }).unwrap(),
                false
            ),
            Ok(())
        );
        assert_matches!(
            check_types_are_compatible(
                &optional_field,
                &syn::parse2(quote! { i32 }).unwrap(),
                false
            ),
            Err(_)
        );
        assert_matches!(
            check_types_are_compatible(
                &required_field,
                &syn::parse2(quote! { Option<i32> }).unwrap(),
                false
            ),
            Err(_)
        );
    }

    #[test]
    fn test_list_validation() {
        let list = FieldType::List(
            Box::new(FieldType::Scalar(Ident::new("").into(), false)),
            false,
        );
        let optional_list = FieldType::List(
            Box::new(FieldType::Scalar(Ident::new("").into(), false)),
            true,
        );
        let option_list_option = FieldType::List(
            Box::new(FieldType::Scalar(Ident::new("").into(), true)),
            true,
        );

        assert_matches!(
            check_types_are_compatible(&list, &syn::parse2(quote! { Vec<i32> }).unwrap(), false),
            Ok(())
        );
        assert_matches!(
            check_types_are_compatible(
                &optional_list,
                &syn::parse2(quote! { Option<Vec<i32>> }).unwrap(),
                false
            ),
            Ok(())
        );
        assert_matches!(
            check_types_are_compatible(
                &option_list_option,
                &syn::parse2(quote! { Option<Vec<Option<i32>>> }).unwrap(),
                false
            ),
            Ok(())
        );
        assert_matches!(
            check_types_are_compatible(&list, &syn::parse2(quote! { i32 }).unwrap(), false),
            Err(_)
        );
        assert_matches!(
            check_types_are_compatible(
                &optional_list,
                &syn::parse2(quote! { Vec<i32> }).unwrap(),
                false
            ),
            Err(_)
        );
        assert_matches!(
            check_types_are_compatible(
                &option_list_option,
                &syn::parse2(quote! { Option<Vec<i32>> }).unwrap(),
                false
            ),
            Err(_)
        );
    }

    #[test]
    fn test_validation_when_flattening() {
        let list = FieldType::List(
            Box::new(FieldType::Scalar(Ident::new("").into(), false)),
            false,
        );
        let optional_list = FieldType::List(
            Box::new(FieldType::Scalar(Ident::new("").into(), false)),
            true,
        );
        let option_list_option = FieldType::List(
            Box::new(FieldType::Scalar(Ident::new("").into(), true)),
            true,
        );

        assert_matches!(
            check_types_are_compatible(
                &option_list_option,
                &syn::parse2(quote! { Vec<i32> }).unwrap(),
                true
            ),
            Ok(())
        );
        assert_matches!(
            check_types_are_compatible(
                &option_list_option,
                &syn::parse2(quote! { Option<Vec<i32>> }).unwrap(),
                true
            ),
            Ok(())
        );
        assert_matches!(
            check_types_are_compatible(
                &optional_list,
                &syn::parse2(quote! { Vec<i32> }).unwrap(),
                true
            ),
            Ok(())
        );

        assert_matches!(
            check_types_are_compatible(
                &list,
                &syn::parse2(quote! { Vec<Option<i32>> }).unwrap(),
                true
            ),
            Err(_)
        );
        assert_matches!(
            check_types_are_compatible(
                &list,
                &syn::parse2(quote! { Option<Vec<i32>> }).unwrap(),
                true
            ),
            Err(_)
        );
    }
}
