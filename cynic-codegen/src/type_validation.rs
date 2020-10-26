use crate::FieldType;

#[derive(Debug, PartialEq, Eq)]
pub enum CheckMode {
    Normal,
    Flattening,
    Recursing,
}

pub fn check_types_are_compatible(
    gql_type: &FieldType,
    rust_type: &syn::Type,
    mode: CheckMode,
) -> Result<(), syn::Error> {
    match mode {
        CheckMode::Normal | CheckMode::Flattening => normal_check(gql_type, rust_type, true),
        CheckMode::Recursing => recursing_check(gql_type, rust_type),
    }
}

fn normal_check(
    gql_type: &FieldType,
    rust_type: &syn::Type,
    flattening: bool,
) -> Result<(), syn::Error> {
    use quote::quote;
    use syn::spanned::Spanned;

    let parsed_type = parse_type(rust_type);

    match parsed_type {
        ParsedType::Unknown => {
            return Err(syn::Error::new(
                rust_type.span(),
                "Cynic does not understand this type. Only un-parameterised types, Vecs, Options & Box are accepted currently.",
            ))
        },
        ParsedType::Box(inner) => {
            // Box is a transparent container for the purposes of checking compatability
            // so just recurse
            return normal_check(gql_type, inner, flattening);
        }
        _ => {}
    }

    if gql_type.is_nullable() {
        if let ParsedType::Optional(inner) = parsed_type {
            return normal_check(&gql_type.as_required(), &inner, flattening);
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
            return normal_check(&item_type, &inner, flattening);
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

fn recursing_check(gql_type: &FieldType, rust_type: &syn::Type) -> Result<(), syn::Error> {
    use quote::quote;
    use syn::spanned::Spanned;

    let parsed_type = parse_type(rust_type);

    if let ParsedType::Unknown = parsed_type {
        return Err(syn::Error::new(
                rust_type.span(),
                "Cynic does not understand this type. Only un-parameterised types, Vecs, Options & Box are accepted currently.",
            ));
    };

    // TODO: Probably still need to care about box...

    if gql_type.is_nullable() {
        // If the field is nullable then we just defer to the normal checks.
        return normal_check(gql_type, rust_type, false);
    };

    if let ParsedType::Optional(inner_rust_type) = parsed_type {
        normal_check(gql_type, inner_rust_type, false)
    } else {
        Err(syn::Error::new(
            rust_type.span(),
            "Recursive types must be wrapped in Option.  Did you mean Option<{}>",
        ))
    }

    /*
    // TODO: I might actually need to take the optional status of fields into account here...

    match (gql_type, parsed_type) {
        (list_field @ FieldType::List(_, _), ParsedType::Optional(inner_rust_type)) => {
            normal_check(list_field, inner_rust_type, false)
        }
        (FieldType::List(_, _), _) => Err(syn::Error::new(
            rust_type.span(),
            format!(
                "Recursive lists must be wrapped in Option.  Did you mean Option<{}>",
                quote! { #rust_type }
            ),
        )),
        (FieldType::Scalar(_, _), _) => Err(syn::Error::new(
            rust_type.span(),
            format!("Scalar fields can't be recursive.  Remove the #[cynic(recurse)] attribute"),
        )),
        (FieldType::Enum(_, _), _) => Err(syn::Error::new(
            rust_type.span(),
            format!("Enum fields can't be recursive.  Remove the #[cynic(recurse)] attribute"),
        )),
        (FieldType::InputObject(_, _), _) => panic!("An input object is inside a normal object"),
        (other_field_type, ParsedType::Optional(inner_rust_type)) => {
            let parsed_inner_type = parse_type(inner_rust_type);
            if let ParsedType::Box(inner_rust_type) = parsed_inner_type {
                normal_check(other_field_type, inner_rust_type, false)
            } else {
                Err(syn::Error::new(
                    rust_type.span(),
                    format!(
                        "Recursive fields must be optional and boxed. Did you mean Option<Box<{}>>",
                        quote! { #inner_rust_type }
                    ),
                ))
            }
        }
        (_, ParsedType::Box(_)) => {
            Err(syn::Error::new(
                rust_type.span(),
                format!(
                    "Recursive fields must be optional and boxed. Did you mean Option<{}>",
                    quote! { #rust_type }
                )
            ))

        }
        (_, ParsedType::List(inner)) => {
            Err(syn::Error::new(
                        rust_type.span(),
                        format!(
                            "This GraphQL type is not a list but you're wrapping the type in Vec.  Did you mean {}",
                            quote! { #inner }
                        )
                    ))
        }
        (_, _) => {
            Err(syn::Error::new(
                rust_type.span(),
                format!("Recursive fields must be optional and boxed. Try wrapping this type in Option<Box<>>")
            ))
        }
    }
    */
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
                if let Some(inner_type) = extract_generic_argument(last_segment) {
                    return ParsedType::Box(inner_type);
                }

                return ParsedType::Unknown;
            }
            if last_segment.ident.to_string() == "Option" {
                if let Some(inner_type) = extract_generic_argument(last_segment) {
                    return ParsedType::Optional(inner_type);
                }

                return ParsedType::Unknown;
            } else if last_segment.ident.to_string() == "Vec" {
                if let Some(inner_type) = extract_generic_argument(last_segment) {
                    return ParsedType::List(inner_type);
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

/// Takes a PathSegment like `Vec<T>` and extracts the `T`
fn extract_generic_argument<'a>(segment: &'a syn::PathSegment) -> Option<&'a syn::Type> {
    if let syn::PathArguments::AngleBracketed(angle_bracketed) = &segment.arguments {
        for arg in &angle_bracketed.args {
            if let syn::GenericArgument::Type(inner_type) = arg {
                return Some(inner_type);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FieldType, Ident};

    use assert_matches::assert_matches;
    use quote::quote;
    use rstest::rstest;
    use syn::parse_quote;

    #[test]
    fn test_required_validation() {
        let required_field = FieldType::Scalar(Ident::new("").into(), false);
        let optional_field = FieldType::Scalar(Ident::new("").into(), true);

        assert_matches!(
            check_types_are_compatible(
                &required_field,
                &syn::parse2(quote! { i32 }).unwrap(),
                CheckMode::Normal
            ),
            Ok(())
        );
        assert_matches!(
            check_types_are_compatible(
                &optional_field,
                &syn::parse2(quote! { Option<i32> }).unwrap(),
                CheckMode::Normal
            ),
            Ok(())
        );
        assert_matches!(
            check_types_are_compatible(
                &optional_field,
                &syn::parse2(quote! { i32 }).unwrap(),
                CheckMode::Normal
            ),
            Err(_)
        );
        assert_matches!(
            check_types_are_compatible(
                &required_field,
                &syn::parse2(quote! { Option<i32> }).unwrap(),
                CheckMode::Normal
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
            check_types_are_compatible(
                &list,
                &syn::parse2(quote! { Vec<i32> }).unwrap(),
                CheckMode::Normal
            ),
            Ok(())
        );
        assert_matches!(
            check_types_are_compatible(
                &optional_list,
                &syn::parse2(quote! { Option<Vec<i32>> }).unwrap(),
                CheckMode::Normal
            ),
            Ok(())
        );
        assert_matches!(
            check_types_are_compatible(
                &option_list_option,
                &syn::parse2(quote! { Option<Vec<Option<i32>>> }).unwrap(),
                CheckMode::Normal
            ),
            Ok(())
        );
        assert_matches!(
            check_types_are_compatible(
                &list,
                &syn::parse2(quote! { i32 }).unwrap(),
                CheckMode::Normal
            ),
            Err(_)
        );
        assert_matches!(
            check_types_are_compatible(
                &optional_list,
                &syn::parse2(quote! { Vec<i32> }).unwrap(),
                CheckMode::Normal
            ),
            Err(_)
        );
        assert_matches!(
            check_types_are_compatible(
                &option_list_option,
                &syn::parse2(quote! { Option<Vec<i32>> }).unwrap(),
                CheckMode::Normal
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
                CheckMode::Flattening
            ),
            Ok(())
        );
        assert_matches!(
            check_types_are_compatible(
                &option_list_option,
                &syn::parse2(quote! { Option<Vec<i32>> }).unwrap(),
                CheckMode::Flattening
            ),
            Ok(())
        );
        assert_matches!(
            check_types_are_compatible(
                &optional_list,
                &syn::parse2(quote! { Vec<i32> }).unwrap(),
                CheckMode::Flattening
            ),
            Ok(())
        );

        assert_matches!(
            check_types_are_compatible(
                &list,
                &syn::parse2(quote! { Vec<Option<i32>> }).unwrap(),
                CheckMode::Flattening
            ),
            Err(_)
        );
        assert_matches!(
            check_types_are_compatible(
                &list,
                &syn::parse2(quote! { Option<Vec<i32>> }).unwrap(),
                CheckMode::Flattening
            ),
            Err(_)
        );
    }

    #[rstest(graphql_field, rust_field,
        case::required_t(
            FieldType::Scalar(Ident::new("T").into(), false),
            parse_quote! { Option<Box<T>> }
        ),

        case::optional_t(
            FieldType::Scalar(Ident::new("T").into(), true),
            parse_quote! { Option<Option<Box<T>>> }
        ),
        case::optional_t_with_simplified(
            FieldType::Scalar(Ident::new("T").into(), true),
            parse_quote! { Option<Box<T>> }
        ),

        case::option_vec_required_t(
            FieldType::List(
                Box::new(FieldType::Scalar(Ident::new("T").into(), false)),
                true
            ),
            parse_quote! { Option<Option<Vec<T>> }
        ),
        case::option_vec_required_t_simplified(
            FieldType::List(
                Box::new(FieldType::Scalar(Ident::new("T").into(), false)),
                true
            ),
            parse_quote! { Option<Vec<T>> }
        ),

        case::required_vec_required_t(
            FieldType::List(
                Box::new(FieldType::Scalar(Ident::new("T").into(), false)),
                false
            ),
            parse_quote! { Option<Vec<T>> }
        ),
        case::required_vec_required_t_simplified(
            FieldType::List(
                Box::new(FieldType::Scalar(Ident::new("T").into(), false)),
                false
            ),
            parse_quote! { Vec<T> }
        ),

        case::required_vec_optional_t(
            FieldType::List(
                Box::new(FieldType::Scalar(Ident::new("T").into(), true)),
                false
            ),
            parse_quote! { Option<Vec<Option<T>>> }
        ),
        case::required_vec_optional_t_simplified(
            FieldType::List(
                Box::new(FieldType::Scalar(Ident::new("T").into(), true)),
                false
            ),
            parse_quote! { Vec<Option<T>> }
        ),
    )]
    fn test_recurse_validation_ok(graphql_field: FieldType, rust_field: syn::Type) {
        assert_matches!(
            check_types_are_compatible(&graphql_field, &rust_field, CheckMode::Recursing),
            Ok(())
        );
    }

    #[rstest(graphql_field, rust_field,
        case::required_t_box(
            FieldType::Scalar(Ident::new("T").into(), false),
            parse_quote! { Box<T> }
        ),
        case::required_t_standalone(
            FieldType::Scalar(Ident::new("T").into(), false),
            parse_quote! { T }
        ),

        case::optional_t_option(
            FieldType::Scalar(Ident::new("T").into(), true),
            parse_quote! { Option<T> }
        ),
        case::optional_t_standalone(
            FieldType::Scalar(Ident::new("T").into(), true),
            parse_quote! { T }
        ),
        case::optional_t_box(
            FieldType::Scalar(Ident::new("T").into(), true),
            parse_quote! { Box<T> }
        ),

        case::option_vec_required_t(
            FieldType::List(
                Box::new(FieldType::Scalar(Ident::new("T").into(), false)),
                true
            ),
            parse_quote! { Vec<T> }
        ),
        case::option_vec_required_t(
            FieldType::List(
                Box::new(FieldType::Scalar(Ident::new("T").into(), false)),
                true
            ),
            parse_quote! { Vec<Option<T>> }
        ),

        case::required_vec_required_t(
            FieldType::List(
                Box::new(FieldType::Scalar(Ident::new("T").into(), false)),
                false
            ),
            parse_quote! { Vec<T> }
        ),
        case::required_vec_required_t_no_vec(
            FieldType::List(
                Box::new(FieldType::Scalar(Ident::new("T").into(), false)),
                false
            ),
            parse_quote! { T }
        ),

        case::required_vec_optional_t_no_vec(
            FieldType::List(
                Box::new(FieldType::Scalar(Ident::new("T").into(), true)),
                false
            ),
            parse_quote! { Option<T> }
        ),
        case::required_vec_optional_t_wrong_nesting(
            FieldType::List(
                Box::new(FieldType::Scalar(Ident::new("T").into(), true)),
                false
            ),
            parse_quote! { Option<Vec<T>> }
        ),
    )]
    fn test_recurse_validation_fail(graphql_field: FieldType, rust_field: syn::Type) {
        assert_matches!(
            check_types_are_compatible(&graphql_field, &rust_field, CheckMode::Recursing),
            Err(_)
        );
    }
}
