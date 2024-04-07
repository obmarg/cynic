use std::collections::HashSet;

use proc_macro2::{Ident, Span};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    MetaList, Token,
};

use crate::fragment_derive::arguments::FieldArgumentValue;

use super::super::arguments::{ArgumentLiteral, CynicArguments, FieldArgument};

#[derive(Debug, Clone)]
pub enum FieldDirective {
    Skip(BooleanLiteral),
    Include(BooleanLiteral),
    Other {
        name: Ident,
        arguments: Vec<FieldArgument>,
    },
}

#[derive(Debug, Clone)]
pub enum BooleanLiteral {
    Boolean(bool),
    Variable(proc_macro2::Ident, Span),
}

pub fn directives_from_field_attrs(attrs: &[syn::Attribute]) -> syn::Result<Vec<FieldDirective>> {
    let mut directives = vec![];
    for attr in attrs {
        if attr.path().is_ident("directives") {
            let parsed: FieldDirectiveAttribute = attr.parse_args()?;
            directives.extend(parsed.directives);
        }
    }

    Ok(directives)
}

/// Implements syn::Parse to parse out arguments from the arguments
/// attribute.
#[derive(Debug)]
struct FieldDirectiveAttribute {
    pub directives: Vec<FieldDirective>,
}

impl Parse for FieldDirectiveAttribute {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lists = Punctuated::<MetaList, Token![,]>::parse_terminated(input)?;
        let mut directives = vec![];
        for list in lists {
            let list_span = list.span();
            let arguments = syn::parse2::<CynicArguments>(list.tokens)?.into_inner();

            if list.path.is_ident("skip") {
                let value = validate_if_or_skip(&arguments, list_span)?;
                directives.push(FieldDirective::Skip(value));
            } else if list.path.is_ident("include") {
                let value = validate_if_or_skip(&arguments, list_span)?;
                directives.push(FieldDirective::Include(value));
            } else {
                let Some(name) = list.path.get_ident().cloned() else {
                    return Err(syn::Error::new_spanned(
                        list.path,
                        "malformed directive - could not determine its name",
                    ));
                };
                directives.push(FieldDirective::Other { name, arguments })
            }
        }

        Ok(FieldDirectiveAttribute { directives })
    }
}

fn validate_if_or_skip(
    arguments: &[FieldArgument],
    parent_span: Span,
) -> Result<BooleanLiteral, syn::Error> {
    let mut already_seen = HashSet::new();
    let mut return_value = None;
    for argument in arguments {
        if argument.argument_name != "if" {
            return Err(syn::Error::new(
                argument.argument_name.span(),
                format!("unknown argument: {}", argument.argument_name),
            ));
        }
        if already_seen.contains(&argument.argument_name) {
            return Err(syn::Error::new(
                argument.argument_name.span(),
                "duplicate argument",
            ));
        }
        already_seen.insert(argument.argument_name.clone());
        match &argument.value {
            FieldArgumentValue::Expression(expr) => {
                return Err(syn::Error::new(
                    expr.span(),
                    "unsuppported syntax - use graphql argument syntax here",
                ))
            }
            FieldArgumentValue::Literal(ArgumentLiteral::Literal(syn::Lit::Bool(bool))) => {
                return_value = Some(BooleanLiteral::Boolean(bool.value));
            }
            FieldArgumentValue::Literal(ArgumentLiteral::Variable(ident, span)) => {
                return_value = Some(BooleanLiteral::Variable(ident.clone(), *span));
            }
            FieldArgumentValue::Literal(other) => {
                return Err(syn::Error::new(
                    other.span(),
                    "invalid argument for if: expected a booolean or variable",
                ));
            }
        }
    }

    let Some(return_value) = return_value else {
        return Err(syn::Error::new(parent_span, "missing argument: if"));
    };

    Ok(return_value)
}

#[cfg(test)]
mod test {
    use assert_matches::assert_matches;
    use quote::quote;
    use syn::{parse2, parse_quote};

    use super::*;

    #[test]
    fn test_parsing_skip() {
        let parsed: FieldDirectiveAttribute = parse_quote! { skip(if: true) };

        let directives = parsed.directives;

        assert_eq!(directives.len(), 1);
        assert_matches!(
            directives[0],
            FieldDirective::Skip(BooleanLiteral::Boolean(true))
        );
    }

    #[test]
    fn test_parsing_include() {
        let parsed: FieldDirectiveAttribute = parse_quote! { include(if: false) };

        let directives = parsed.directives;

        assert_eq!(directives.len(), 1);
        assert_matches!(
            directives[0],
            FieldDirective::Include(BooleanLiteral::Boolean(false))
        );
    }

    #[test]
    fn test_parsing_boolean_literal_with_variables() {
        let parsed: FieldDirectiveAttribute = parse_quote! { include(if: $someVariable) };

        let directives = parsed.directives;

        assert_eq!(directives.len(), 1);
        assert_matches!(
            &directives[0],
            FieldDirective::Include(BooleanLiteral::Variable(ident, _)) => {
                assert_eq!(ident, "someVariable");
            }
        );
    }

    #[test]
    fn test_missing_argument() {
        let err = parse2::<FieldDirectiveAttribute>(quote! { include() }).unwrap_err();
        insta::assert_display_snapshot!(err, @"missing argument: if");
    }

    #[test]
    fn test_unknown_argument() {
        let err = parse2::<FieldDirectiveAttribute>(quote! { include(if: true, other: false) })
            .unwrap_err();
        insta::assert_display_snapshot!(err, @"unknown argument: other");
    }

    #[test]
    fn test_duplicate_argument() {
        let err =
            parse2::<FieldDirectiveAttribute>(quote! { include(if: true, if: false) }).unwrap_err();
        insta::assert_display_snapshot!(err, @"duplicate argument");
    }

    #[test]
    fn test_non_boolean_argument() {
        let err = parse2::<FieldDirectiveAttribute>(quote! { include(if: [true]) }).unwrap_err();
        insta::assert_display_snapshot!(err, @"invalid argument for if: expected a booolean or variable");
    }

    #[test]
    fn test_other_directives() {
        let parsed: FieldDirectiveAttribute =
            parse_quote! { other(obj: {name: true}, list: ["hello"]) };

        let directives = parsed.directives;

        assert_eq!(directives.len(), 1);
        assert_matches!(&directives[0], FieldDirective::Other { name, arguments } => {
            assert_eq!(name, "other");
            assert_eq!(arguments.len(), 2);
        });
    }
}
