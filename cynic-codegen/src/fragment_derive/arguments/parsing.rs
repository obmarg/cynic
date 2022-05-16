use proc_macro2::Span;
use syn::{
    ext::IdentExt,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned,
    Ident, Result, Token,
};

pub fn arguments_from_field_attrs(
    attrs: &[syn::Attribute],
) -> Result<Option<(Vec<FieldArgument>, proc_macro2::Span)>> {
    for attr in attrs {
        if attr.path.is_ident("arguments") {
            let parsed: CynicArguments = attr.parse_args()?;
            return Ok(Some((parsed.arguments.into_iter().collect(), attr.span())));
        }
    }
    Ok(None)
}

/// Implements syn::Parse to parse out arguments from the arguments
/// attribute.
#[derive(Debug)]
pub struct CynicArguments {
    pub arguments: Punctuated<FieldArgument, Token![,]>,
}

impl Parse for CynicArguments {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(CynicArguments {
            arguments: Punctuated::parse_terminated(input)?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct FieldArgument {
    pub argument_name: Ident,
    pub value: FieldArgumentValue,
}

#[derive(Debug, Clone)]
pub enum FieldArgumentValue {
    Literal(ArgumentLiteral),
    Expression(Box<syn::Expr>),
}

impl Parse for FieldArgument {
    fn parse(input: ParseStream) -> Result<Self> {
        let argument_name = input.call(Ident::parse_any)?;
        let lookahead = input.lookahead1();
        let value;
        if lookahead.peek(Token![:]) {
            input.parse::<Token![:]>()?;
            value = FieldArgumentValue::Literal(input.parse()?);
        } else if lookahead.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            value = FieldArgumentValue::Expression(input.parse()?);
        } else {
            return Err(lookahead.error());
        }

        Ok(FieldArgument {
            argument_name,
            value,
        })
    }
}

#[derive(Debug, Clone)]
pub enum ArgumentLiteral {
    Literal(syn::Lit),
    Object(Punctuated<FieldArgument, Token![,]>, Span),
    List(Punctuated<ArgumentLiteral, Token![,]>, Span),
    Variable(proc_macro2::Ident, Span),
    Null(Span),
}

impl ArgumentLiteral {
    pub(super) fn span(&self) -> Span {
        match self {
            ArgumentLiteral::Literal(lit) => lit.span(),
            ArgumentLiteral::Object(_, span) => *span,
            ArgumentLiteral::List(_, span) => *span,
            ArgumentLiteral::Variable(_, span) => *span,
            ArgumentLiteral::Null(span) => *span,
        }
    }
}

impl Parse for ArgumentLiteral {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::token::Brace) {
            let span = input.span();
            let content;
            syn::braced!(content in input);

            Ok(ArgumentLiteral::Object(
                content.parse_terminated(FieldArgument::parse)?,
                span,
            ))
        } else if lookahead.peek(syn::token::Bracket) {
            let span = input.span();
            let content;
            syn::bracketed!(content in input);

            Ok(ArgumentLiteral::List(
                content.parse_terminated(ArgumentLiteral::parse)?,
                span,
            ))
        } else if lookahead.peek(Token![$]) {
            let dollar_span = input.span();
            input.parse::<Token![$]>()?;

            let mut span = input.span();
            if let Some(joined_span) = dollar_span.join(span) {
                // This only works on nightly, so fall back to the span of the ident.
                span = joined_span;
            }

            Ok(ArgumentLiteral::Variable(input.parse()?, span))
        } else if lookahead.peek(syn::Lit) {
            Ok(ArgumentLiteral::Literal(input.parse()?))
        } else if lookahead.peek(Ident) {
            let ident = input.call(Ident::parse_any)?;

            if ident == "null" {
                return Ok(ArgumentLiteral::Null(ident.span()));
            }

            Err(syn::Error::new(
                ident.span(),
                format!("Unknown token: {ident}"),
            ))
        } else {
            Err(lookahead.error())
        }
    }
}

#[cfg(test)]
mod test {
    use assert_matches::assert_matches;
    use syn::parse_quote;

    use super::*;

    #[test]
    fn test_parsing_string_literal() {
        let parsed: CynicArguments = parse_quote! { x: "abcd" };

        let arguments = parsed.arguments.iter().collect::<Vec<_>>();

        assert_eq!(arguments.len(), 1);
        assert_eq!(arguments[0].argument_name.to_string(), "x".to_string());
        assert_matches!(
            arguments[0].value,
            FieldArgumentValue::Literal(ArgumentLiteral::Literal(_))
        );
    }

    #[test]
    fn test_parsing_boolean_literal() {
        let parsed: CynicArguments = parse_quote! { x: true, y: false };

        let arguments = parsed.arguments.iter().collect::<Vec<_>>();

        assert_eq!(arguments.len(), 2);
        assert_eq!(arguments[0].argument_name.to_string(), "x".to_string());
        assert_matches!(
            &arguments[0].value,
            FieldArgumentValue::Literal(ArgumentLiteral::Literal(lit)) => {
                let expected: syn::Lit = parse_quote!{ true };
                assert_eq!(lit, &expected);
            }
        );

        assert_eq!(arguments[1].argument_name.to_string(), "y".to_string());
        assert_matches!(
            &arguments[1].value,
            FieldArgumentValue::Literal(ArgumentLiteral::Literal(lit)) => {
                let expected: syn::Lit = parse_quote!{ false };
                assert_eq!(lit, &expected);
            }
        );
    }

    #[test]
    fn test_parsing_null() {
        let parsed: CynicArguments = parse_quote! { x: null };

        let arguments = parsed.arguments.iter().collect::<Vec<_>>();

        assert_eq!(arguments.len(), 1);
        assert_eq!(arguments[0].argument_name.to_string(), "x".to_string());
        assert_matches!(
            &arguments[0].value,
            FieldArgumentValue::Literal(ArgumentLiteral::Null(_))
        );
    }

    #[test]
    fn test_parsing_multiple_arg_expressions() {
        let parsed: CynicArguments = parse_quote! { x: 1, y: $variable };

        let arguments = parsed.arguments.iter().collect::<Vec<_>>();

        assert_eq!(arguments.len(), 2);
        assert_eq!(arguments[0].argument_name.to_string(), "x".to_string());
        assert_matches!(
            arguments[0].value,
            FieldArgumentValue::Literal(ArgumentLiteral::Literal(_))
        );

        assert_eq!(arguments[1].argument_name.to_string(), "y".to_string());
        assert_matches!(&arguments[1].value, FieldArgumentValue::Literal(ArgumentLiteral::Variable(name ,_)) => {
            assert_eq!(name.to_string(), "variable");
        });
    }

    #[test]
    fn test_parsing_list_and_object() {
        let parsed: CynicArguments = parse_quote! { x: {fieldOne: ["hello"], fieldTwo: "hello"}};
        let arguments = parsed.arguments.iter().collect::<Vec<_>>();

        assert_eq!(arguments.len(), 1);
        assert_eq!(arguments[0].argument_name.to_string(), "x".to_string());
        assert_matches!(&arguments[0].value, FieldArgumentValue::Literal(ArgumentLiteral::Object(fields, _)) => {
            let fields = fields.iter().collect::<Vec<_>>();
            assert_eq!(fields.len(), 2);

            assert_eq!(fields[0].argument_name.to_string(), "fieldOne");
            assert_matches!(&fields[0].value, FieldArgumentValue::Literal(ArgumentLiteral::List(vals, _)) => {

                let vals = vals.iter().collect::<Vec<_>>();
                assert_eq!(vals.len(), 1);

                assert_matches!(vals[0], ArgumentLiteral::Literal(_));
            });

            assert_eq!(fields[1].argument_name.to_string(), "fieldTwo");
            assert_matches!(fields[1].value, FieldArgumentValue::Literal(ArgumentLiteral::Literal(_)));
        });
    }
}
