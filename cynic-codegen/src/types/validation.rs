use proc_macro2::Span;
use syn::spanned::Spanned;

use {
    super::parsing::{parse_rust_type, RustType},
    crate::schema::types::{InputType, InputValue, OutputType, TypeRef},
};

#[derive(Debug, PartialEq, Eq)]
pub enum CheckMode {
    OutputTypes,
    Flattening,
    Recursing,
    Spreading,
    Skippable,
    Defaulted,
}

pub fn check_types_are_compatible(
    gql_type: &TypeRef<'_, OutputType<'_>>,
    rust_type: &syn::Type,
    mode: CheckMode,
) -> Result<(), syn::Error> {
    let parsed_rust_type = parse_rust_type(rust_type);
    match mode {
        CheckMode::Flattening => output_type_check(gql_type, &parsed_rust_type, true)?,
        CheckMode::OutputTypes => output_type_check(gql_type, &parsed_rust_type, false)?,
        CheckMode::Recursing => recursing_check(gql_type, &parsed_rust_type)?,
        CheckMode::Skippable => {
            if !outer_type_is_option(rust_type) {
                return Err(TypeValidationError::SkippableFieldWithoutError {
                    provided_type: rust_type.to_string(),
                    span: rust_type.span(),
                }
                .into());
            }
            output_type_check(gql_type, &parsed_rust_type, false)?;
        }
        CheckMode::Defaulted => {
            let gql_type = match gql_type.clone() {
                TypeRef::Nullable(inner_ty) => *inner_ty,
                ty => ty,
            };
            output_type_check(&gql_type, &parsed_rust_type, false)?;
        }
        CheckMode::Spreading => {
            panic!("check_types_are_compatible shouldn't be called with CheckMode::Spreading")
        }
    }

    Ok(())
}

pub fn check_input_types_are_compatible(
    gql_type: &InputValue<'_>,
    rust_type: &syn::Type,
) -> Result<(), syn::Error> {
    let parsed_type = parse_rust_type(rust_type);

    input_type_check(&gql_type.value_type, gql_type.has_default, &parsed_type)?;

    Ok(())
}

pub fn check_spread_type(rust_type: &syn::Type) -> Result<(), syn::Error> {
    fn inner_fn(rust_type: &RustType<'_>) -> Result<(), syn::Error> {
        match rust_type {
            RustType::Ref { inner, .. } => {
                // Box is a transparent container for the purposes of checking compatibility
                // so just recurse
                inner_fn(inner.as_ref())
            }
            RustType::Optional { .. } => Err(TypeValidationError::SpreadOnOption {
                span: rust_type.span(),
            }
            .into()),
            RustType::List { .. } => Err(TypeValidationError::SpreadOnVec {
                span: rust_type.span(),
            }
            .into()),
            RustType::SimpleType { .. } => {
                // No way to tell if the given type is actually compatible,
                // but the rust compiler should help us with that.
                Ok(())
            }
        }
    }

    inner_fn(&parse_rust_type(rust_type))
}

/// Checks if the type is `Option`, `&Option`, `Box<Option>` etc.
pub fn outer_type_is_option(rust_type: &syn::Type) -> bool {
    fn inner_fn(rust_type: &RustType<'_>) -> bool {
        match rust_type {
            RustType::Optional { .. } => true,
            RustType::List { .. } => false,
            RustType::Ref { inner, .. } => inner_fn(inner.as_ref()),
            RustType::SimpleType { .. } => false,
        }
    }

    inner_fn(&parse_rust_type(rust_type))
}

fn output_type_check<'a>(
    gql_type: &TypeRef<'a, OutputType<'a>>,
    rust_type: &RustType<'a>,
    flattening: bool,
) -> Result<(), TypeValidationError> {
    match (&gql_type, rust_type) {
        (_, RustType::Ref { inner, .. }) => {
            // Box is a transparent container for the purposes of checking compatibility
            // so just recurse
            output_type_check(gql_type, inner.as_ref(), flattening)
        }
        (TypeRef::Nullable(inner_gql), RustType::Optional { inner, .. }) => {
            output_type_check(inner_gql, inner.as_ref(), flattening)
        }
        (TypeRef::Nullable(inner_gql), _) if flattening => {
            // If we're flattening then we should still check the inner types line up...
            output_type_check(inner_gql, rust_type, flattening)
        }
        (TypeRef::Nullable(_), _) => Err(TypeValidationError::FieldIsOptional {
            provided_type: rust_type.to_syn().to_string(),

            span: rust_type.span(),
        }),
        (gql_type, RustType::Optional { inner, .. }) => {
            // It should be fine for an output field to be `Option` if the schema
            // type isn't nullable.  it's pointless, but won't crash so
            // we just need to check the inner types
            output_type_check(gql_type, inner.as_ref(), flattening)
        }
        (TypeRef::List(item_type), RustType::List { inner, .. }) => {
            output_type_check(item_type.as_ref(), inner.as_ref(), flattening)
        }
        (TypeRef::List(_), _) => {
            // If the server is going to return a list we can't not have a Vec here...
            Err(TypeValidationError::FieldIsList {
                provided_type: rust_type.to_syn().to_string(),
                span: rust_type.span(),
            })
        }
        (_, RustType::List { inner, .. }) => Err(TypeValidationError::FieldIsNotList {
            provided_type: inner.to_syn().to_string(),
            span: rust_type.span(),
        }),
        (TypeRef::Named(_, _), RustType::SimpleType { .. }) => Ok(()),
    }
}

fn input_type_check<'a>(
    gql_type: &TypeRef<'a, InputType<'a>>,
    has_default: bool,
    rust_type: &RustType<'_>,
) -> Result<(), TypeValidationError> {
    match (&gql_type, rust_type) {
        (gql_type, RustType::Ref { inner, .. }) => {
            // Box is a transparent container for the purposes of checking compatibility
            // so just recurse
            input_type_check(gql_type, has_default, inner.as_ref())
        }
        (TypeRef::Nullable(inner_gql), RustType::Optional { inner, .. }) => {
            input_type_check(inner_gql, false, inner.as_ref())
        }
        (TypeRef::Nullable(inner_gql), _) => {
            // For input types its fine if a field isn't actually optional.
            // We just need to check that the inner types line up.
            input_type_check(inner_gql, false, rust_type)
        }
        (_, RustType::Optional { inner, .. }) if has_default => {
            // If an input type is required but has a default then
            // it's ok for it to be wrapped in option.
            input_type_check(gql_type, false, inner)
        }
        (_, RustType::Optional { inner, .. }) => Err(TypeValidationError::FieldIsRequired {
            provided_type: inner.to_syn().to_string(),
            span: rust_type.span(),
        }),
        (TypeRef::List(item_type), RustType::List { inner, .. }) => {
            input_type_check(item_type.as_ref(), false, inner)
        }
        (TypeRef::List(item_type), _) => {
            // For input types its fine to provide a single item instead of a list.
            // We just need to check that the inner types line up.
            input_type_check(item_type, false, rust_type)
        }
        (_, RustType::List { inner, .. }) => Err(TypeValidationError::FieldIsNotList {
            provided_type: inner.to_syn().to_string(),
            span: rust_type.span(),
        }),
        (TypeRef::Named(_, _), RustType::SimpleType { .. }) => Ok(()),
    }
}

fn recursing_check(
    gql_type: &TypeRef<'_, OutputType<'_>>,
    rust_type: &RustType<'_>,
) -> Result<(), TypeValidationError> {
    if let TypeRef::Nullable(_) = gql_type {
        // If the field is nullable then we just defer to the normal checks.
        return output_type_check(gql_type, rust_type, false);
    };

    if let RustType::Optional { inner, .. } = rust_type {
        output_type_check(gql_type, inner.as_ref(), false)
    } else {
        Err(TypeValidationError::RecursiveFieldWithoutOption {
            provided_type: rust_type.to_syn().to_string(),
            span: rust_type.span(),
        })
    }
}

#[derive(Debug)]
enum TypeValidationError {
    FieldIsOptional { provided_type: String, span: Span },
    FieldIsRequired { provided_type: String, span: Span },
    FieldIsList { provided_type: String, span: Span },
    FieldIsNotList { provided_type: String, span: Span },
    RecursiveFieldWithoutOption { provided_type: String, span: Span },
    SpreadOnOption { span: Span },
    SpreadOnVec { span: Span },
    SkippableFieldWithoutError { provided_type: String, span: Span },
}

impl From<TypeValidationError> for syn::Error {
    fn from(err: TypeValidationError) -> Self {
        let span = err.span();
        let message = match err {
            TypeValidationError::FieldIsOptional { provided_type, .. } =>
                format!("This field is nullable but you're not wrapping the type in MaybeUndefined or Option.  Did you mean cynic::MaybeUndefined<{}> or Option<{}>", provided_type, provided_type),
            TypeValidationError::FieldIsRequired { provided_type, .. } =>
                format!("This field is not nullable but you're wrapping the type in Option or MaybeUndefined.  Did you mean {}", provided_type),
            TypeValidationError::FieldIsList { provided_type, .. } => {
                format!("This field is a list but you're not wrapping the type in Vec.  Did you mean Vec<{}>", provided_type)
            },
            TypeValidationError::FieldIsNotList { provided_type, .. } => {
                format!("This field is not a list but you're wrapping the type in Vec.  Did you mean {}", provided_type)
            },
            TypeValidationError::RecursiveFieldWithoutOption { provided_type, .. } => {
                format!("Recursive types must be wrapped in MaybeUndefined or Option.  Did you mean cynic::MaybeUndefined<{}> or Option<{}>", provided_type, provided_type)
            }
            TypeValidationError::SpreadOnOption { .. } => "You can't spread on an optional field".to_string(),
            TypeValidationError::SpreadOnVec { .. } => "You can't spread on a list field".to_string(),
            TypeValidationError::SkippableFieldWithoutError { provided_type,.. } => format!("This field has @skip or @include on it so it must be optional.  Did you mean Option<{provided_type}>"),
        };

        syn::Error::new(span, message)
    }
}

impl From<TypeValidationError> for crate::Errors {
    fn from(err: TypeValidationError) -> Self {
        crate::Errors::from(syn::Error::from(err))
    }
}

impl TypeValidationError {
    fn span(&self) -> Span {
        match self {
            TypeValidationError::FieldIsOptional { span, .. } => *span,
            TypeValidationError::FieldIsRequired { span, .. } => *span,
            TypeValidationError::FieldIsList { span, .. } => *span,
            TypeValidationError::FieldIsNotList { span, .. } => *span,
            TypeValidationError::RecursiveFieldWithoutOption { span, .. } => *span,
            TypeValidationError::SpreadOnOption { span } => *span,
            TypeValidationError::SpreadOnVec { span } => *span,
            TypeValidationError::SkippableFieldWithoutError { span, .. } => *span,
        }
    }
}

trait SynTypeExt {
    fn to_string(&self) -> String;
}

impl SynTypeExt for syn::Type {
    fn to_string(&self) -> String {
        quote::quote! { #self }.to_string().replace(' ', "")
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use {
        super::*,
        crate::schema::types::{InputType, OutputType},
    };

    use {assert_matches::assert_matches, quote::quote, rstest::rstest, syn::parse_quote};

    type OutputTypeRef<'a> = super::TypeRef<'a, OutputType<'a>>;
    type InputTypeRef<'a> = super::TypeRef<'a, InputType<'a>>;

    fn call_output_type_check(
        gql_type: &OutputTypeRef<'_>,
        rust_type: &syn::Type,
        flattening: bool,
    ) -> Result<(), TypeValidationError> {
        output_type_check(gql_type, &parse_rust_type(rust_type), flattening)
    }

    #[test]
    fn test_output_type_check() {
        let required_field = TypeRef::Named("test".into(), PhantomData);
        let optional_field = TypeRef::Nullable(Box::new(required_field.clone()));

        assert_matches!(
            call_output_type_check(
                &required_field,
                &syn::parse2(quote! { i32 }).unwrap(),
                false
            ),
            Ok(())
        );
        assert_matches!(
            call_output_type_check(
                &required_field,
                &syn::parse2(quote! { DateTime<Utc> }).unwrap(),
                false
            ),
            Ok(())
        );
        assert_matches!(
            call_output_type_check(
                &optional_field,
                &syn::parse2(quote! { Option<i32> }).unwrap(),
                false
            ),
            Ok(())
        );
        assert_matches!(
            call_output_type_check(
                &optional_field,
                &syn::parse2(quote! { i32 }).unwrap(),
                false
            ),
            Err(TypeValidationError::FieldIsOptional {
                provided_type,
                ..
            }) => {
                assert_eq!(provided_type, "i32")
            }
        );
        assert_matches!(
            call_output_type_check(
                &required_field,
                &syn::parse2(quote! { Option<i32> }).unwrap(),
                false
            ),
            Ok(())
        );
    }

    #[test]
    fn test_output_type_list_validation() {
        let named = TypeRef::Named("test".into(), PhantomData);
        let list = TypeRef::List(Box::new(named.clone()));
        let optional_list = TypeRef::Nullable(Box::new(TypeRef::List(Box::new(named.clone()))));
        let option_list_option = TypeRef::Nullable(Box::new(TypeRef::List(Box::new(
            TypeRef::Nullable(Box::new(named.clone())),
        ))));

        assert_matches!(
            call_output_type_check(&list, &syn::parse2(quote! { Vec<i32> }).unwrap(), false),
            Ok(())
        );
        assert_matches!(
            call_output_type_check(
                &optional_list,
                &syn::parse2(quote! { Option<Vec<i32>> }).unwrap(),
                false
            ),
            Ok(())
        );
        assert_matches!(
            call_output_type_check(
                &option_list_option,
                &syn::parse2(quote! { Option<Vec<Option<i32>>> }).unwrap(),
                false
            ),
            Ok(())
        );
        assert_matches!(
            call_output_type_check(
                &list,
                &syn::parse2(quote! { Vec<DateTime<Utc>> }).unwrap(),
                false
            ),
            Ok(())
        );
        assert_matches!(
            call_output_type_check(
                &optional_list,
                &syn::parse2(quote! { Option<Vec<DateTime<Utc>>> }).unwrap(),
                false
            ),
            Ok(())
        );
        assert_matches!(
            call_output_type_check(
                &option_list_option,
                &syn::parse2(quote! { Option<Vec<Option<DateTime<Utc>>>> }).unwrap(),
                false
            ),
            Ok(())
        );
        assert_matches!(
            call_output_type_check(
                &list,
                &syn::parse2(quote! { Option<Vec<i32>> }).unwrap(),
                false
            ),
            Ok(())
        );
        assert_matches!(
            call_output_type_check(&list, &syn::parse2(quote! { i32 }).unwrap(), false),
            Err(TypeValidationError::FieldIsList { provided_type, .. }) => {
                assert_eq!(provided_type, "i32")
            }
        );
        assert_matches!(
            call_output_type_check(
                &optional_list,
                &syn::parse2(quote! { Vec<i32> }).unwrap(),
                false
            ),
            Err(TypeValidationError::FieldIsOptional { provided_type, .. }) => {
                assert_eq!(provided_type, "Vec<i32>")
            }
        );
        assert_matches!(
            call_output_type_check(
                &option_list_option,
                &syn::parse2(quote! { Option<Vec<i32>> }).unwrap(),
                false
            ),
            Err(TypeValidationError::FieldIsOptional { provided_type, .. }) => {
                assert_eq!(provided_type, "i32")
            }
        );
        assert_matches!(
            call_output_type_check(
                &option_list_option,
                &syn::parse2(quote! { Option<DateTime<Vec<Option<i32>>>> }).unwrap(),
                false
            ),
            Err(_)
        );
    }

    #[test]
    fn test_validation_when_flattening() {
        let named = TypeRef::Named("test".into(), PhantomData);
        let list = TypeRef::List(Box::new(named.clone()));
        let optional_list = TypeRef::Nullable(Box::new(TypeRef::List(Box::new(named.clone()))));
        let option_list_option = TypeRef::Nullable(Box::new(TypeRef::List(Box::new(
            TypeRef::Nullable(Box::new(named.clone())),
        ))));

        assert_matches!(
            call_output_type_check(
                &option_list_option,
                &syn::parse2(quote! { Vec<i32> }).unwrap(),
                true
            ),
            Ok(())
        );
        assert_matches!(
            call_output_type_check(
                &option_list_option,
                &syn::parse2(quote! { Option<Vec<i32>> }).unwrap(),
                true
            ),
            Ok(())
        );
        assert_matches!(
            call_output_type_check(
                &optional_list,
                &syn::parse2(quote! { Vec<i32> }).unwrap(),
                true
            ),
            Ok(())
        );
        assert_matches!(
            call_output_type_check(
                &list,
                &syn::parse2(quote! { Vec<Option<i32>> }).unwrap(),
                true
            ),
            Ok(())
        );
        assert_matches!(
            call_output_type_check(
                &list,
                &syn::parse2(quote! { Option<Vec<i32>> }).unwrap(),
                true
            ),
            Ok(())
        );
        assert_matches!(
            call_output_type_check(&list, &syn::parse2(quote! { Option<i32> }).unwrap(), true),
            Err(TypeValidationError::FieldIsList { provided_type, .. }) => {
                assert_eq!(provided_type, "i32")
            }
        );
    }

    fn call_input_type_check(
        gql_type: &InputTypeRef<'_>,
        has_default: bool,
        rust_type: &syn::Type,
    ) -> Result<(), TypeValidationError> {
        input_type_check(gql_type, has_default, &parse_rust_type(rust_type))
    }

    #[test]
    fn test_input_type_validation() {
        let required_field = TypeRef::Named("test".into(), PhantomData);
        let optional_field = TypeRef::Nullable(Box::new(required_field.clone()));

        assert_matches!(
            call_input_type_check(
                &required_field,
                false,
                &syn::parse2(quote! { i32 }).unwrap(),
            ),
            Ok(())
        );
        assert_matches!(
            call_input_type_check(
                &required_field,
                false,
                &syn::parse2(quote! { DateTime<Utc> }).unwrap(),
            ),
            Ok(())
        );
        assert_matches!(
            call_input_type_check(
                &optional_field,
                false,
                &syn::parse2(quote! { Option<i32> }).unwrap(),
            ),
            Ok(())
        );
        assert_matches!(
            call_input_type_check(
                &optional_field,
                false,
                &syn::parse2(quote! { i32 }).unwrap(),
            ),
            Ok(())
        );
        assert_matches!(
            call_input_type_check(
                &required_field,
                false,
                &syn::parse2(quote! { Option<i32> }).unwrap(),
            ),
            Err(TypeValidationError::FieldIsRequired {provided_type, ..}) => {
                assert_eq!(provided_type, "i32")
            }
        );
    }

    #[test]
    fn test_input_type_validation_with_default() {
        let required_field = TypeRef::Named("test".into(), PhantomData);
        let optional_field = TypeRef::Nullable(Box::new(required_field.clone()));

        assert_matches!(
            call_input_type_check(&required_field, true, &syn::parse2(quote! { i32 }).unwrap(),),
            Ok(())
        );
        assert_matches!(
            call_input_type_check(
                &required_field,
                true,
                &syn::parse2(quote! { DateTime<Utc> }).unwrap(),
            ),
            Ok(())
        );
        assert_matches!(
            call_input_type_check(
                &optional_field,
                true,
                &syn::parse2(quote! { Option<i32> }).unwrap(),
            ),
            Ok(())
        );
        assert_matches!(
            call_input_type_check(&optional_field, true, &syn::parse2(quote! { i32 }).unwrap(),),
            Ok(())
        );
        assert_matches!(
            call_input_type_check(
                &required_field,
                true,
                &syn::parse2(quote! { Option<i32> }).unwrap(),
            ),
            Ok(())
        );
    }

    #[test]
    fn test_input_type_list_validation() {
        let named = TypeRef::Named("test".into(), PhantomData);
        let list = TypeRef::List(Box::new(named.clone()));
        let optional_list = TypeRef::Nullable(Box::new(TypeRef::List(Box::new(named.clone()))));
        let option_list_option = TypeRef::Nullable(Box::new(TypeRef::List(Box::new(
            TypeRef::Nullable(Box::new(named.clone())),
        ))));

        assert_matches!(
            call_input_type_check(&list, false, &syn::parse2(quote! { Vec<i32> }).unwrap(),),
            Ok(())
        );
        assert_matches!(
            call_input_type_check(
                &optional_list,
                false,
                &syn::parse2(quote! { Option<Vec<i32>> }).unwrap(),
            ),
            Ok(())
        );
        assert_matches!(
            call_input_type_check(
                &option_list_option,
                false,
                &syn::parse2(quote! { Option<Vec<Option<i32>>> }).unwrap(),
            ),
            Ok(())
        );
        assert_matches!(
            call_input_type_check(
                &list,
                false,
                &syn::parse2(quote! { Vec<DateTime<Utc>> }).unwrap(),
            ),
            Ok(())
        );
        assert_matches!(
            call_input_type_check(
                &optional_list,
                false,
                &syn::parse2(quote! { Option<Vec<DateTime<Utc>>> }).unwrap(),
            ),
            Ok(())
        );
        assert_matches!(
            call_input_type_check(
                &option_list_option,
                false,
                &syn::parse2(quote! { Option<Vec<Option<DateTime<Utc>>>> }).unwrap(),
            ),
            Ok(())
        );
        assert_matches!(
            call_input_type_check(&list, false, &syn::parse2(quote! { i32 }).unwrap(),),
            Ok(())
        );
        assert_matches!(
            call_input_type_check(&optional_list, false, &syn::parse2(quote! { i32 }).unwrap(),),
            Ok(())
        );
        assert_matches!(
            call_input_type_check(
                &optional_list,
                false,
                &syn::parse2(quote! { Vec<i32> }).unwrap(),
            ),
            Ok(())
        );
        assert_matches!(
            call_input_type_check(
                &option_list_option,
                false,
                &syn::parse2(quote! { Option<Vec<i32>> }).unwrap(),
            ),
            Ok(())
        );
        assert_matches!(
            call_input_type_check(
                &option_list_option,
                false,
                &syn::parse2(quote! { Option<Vec<Inner<'a>>> }).unwrap(),
            ),
            Ok(())
        );
    }

    #[rstest(graphql_field, rust_field,
        case::required_t(
            TypeRef::Named("T".into(), PhantomData),
            parse_quote! { Option<Box<T>> }
        ),

        case::optional_t(
            TypeRef::Nullable(Box::new(TypeRef::Named("T".into(), PhantomData))),
            parse_quote! { Option<T> }
        ),

        case::option_vec_required_t(
            TypeRef::Nullable(Box::new(
                TypeRef::List(Box::new(TypeRef::Named("T".into(), PhantomData)))
            )),
            parse_quote! { Option<Vec<T>> }
        ),

        case::required_vec_required_t(
            TypeRef::List(Box::new(TypeRef::Named("T".into(), PhantomData))),
            parse_quote! { Option<Vec<T>> }
        ),
    )]
    fn test_recurse_validation_ok(graphql_field: OutputTypeRef<'_>, rust_field: syn::Type) {
        assert_matches!(
            check_types_are_compatible(&graphql_field, &rust_field, CheckMode::Recursing),
            Ok(())
        );
    }

    #[rstest(graphql_field, rust_field,
        case::required_t_box(
            TypeRef::Named("T".into(), PhantomData),
            parse_quote! { Box<T> }
        ),
        case::required_t_standalone(
            TypeRef::Named("T".into(), PhantomData),
            parse_quote! { T }
        ),

        case::optional_t_standalone(
            TypeRef::Nullable(Box::new(TypeRef::Named("T".into(), PhantomData))),
            parse_quote! { T }
        ),
        case::optional_t_box(
            TypeRef::Nullable(Box::new(TypeRef::Named("T".into(), PhantomData))),
            parse_quote! { Box<T> }
        ),

        case::option_vec_required_t(
            TypeRef::Nullable(Box::new(
                TypeRef::List(Box::new(TypeRef::Named("T".into(), PhantomData)))
            )),
            parse_quote! { Vec<T> }
        ),
        case::option_vec_required_t(
            TypeRef::Nullable(Box::new(
                TypeRef::List(Box::new(TypeRef::Named("T".into(), PhantomData)))
            )),
            parse_quote! { Vec<Option<T>> }
        ),

        case::required_vec_required_t(
            TypeRef::List(Box::new(TypeRef::Named("T".into(), PhantomData))),
            parse_quote! { Vec<T> }
        ),
        case::required_vec_required_t_no_vec(
            TypeRef::List(Box::new(TypeRef::Named("T".into(), PhantomData))),
            parse_quote! { T }
        ),

        case::required_vec_optional_t_no_vec(
            TypeRef::List(Box::new(
                TypeRef::Nullable(Box::new(TypeRef::Named("T".into(), PhantomData)))
            )),
            parse_quote! { Option<T> }
        ),
        case::required_vec_optional_t_wrong_nesting(
            TypeRef::List(Box::new(
                TypeRef::Nullable(Box::new(TypeRef::Named("T".into(), PhantomData)))
            )),
            parse_quote! { Option<Vec<T>> }
        ),
    )]
    fn test_recurse_validation_fail(graphql_field: OutputTypeRef<'_>, rust_field: syn::Type) {
        assert_matches!(
            check_types_are_compatible(&graphql_field, &rust_field, CheckMode::Recursing),
            Err(_)
        );
    }
}
