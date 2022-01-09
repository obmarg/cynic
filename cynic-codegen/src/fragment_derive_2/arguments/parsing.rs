use proc_macro2::{Span, TokenStream};
use syn::{
    ext::IdentExt,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Expr, Ident, Result, Token,
};

pub fn arguments_from_field_attrs(attrs: &[syn::Attribute]) -> Result<Vec<FieldArgument>> {
    for attr in attrs {
        if attr.path.is_ident("arguments") {
            let parsed: CynicArguments = attr.parse_args()?;
            return Ok(parsed.arguments.into_iter().collect());
        }
    }
    Ok(vec![])
}

/// Implements syn::Parse to parse out arguments from the arguments
/// attribute.
#[derive(Debug)]
pub struct CynicArguments {
    // TODO: technically we want some kind of MaybePunctuated.
    // Worth looking into that later
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
    pub value: ArgumentLiteral,
}

impl Parse for FieldArgument {
    fn parse(input: ParseStream) -> Result<Self> {
        let argument_name = input.parse()?;
        input.parse::<Token![:]>()?;
        let value = input.parse()?;

        Ok(FieldArgument {
            argument_name,
            value,
        })
    }
}

// TODO: Think about spans here...
#[derive(Debug, Clone)]
pub enum ArgumentLiteral {
    // True & false are idents aren't they, fuck
    Bool(bool, Span),
    Literal(syn::Lit),
    Object(Punctuated<FieldArgument, Token![,]>, Span),
    List(Punctuated<ArgumentLiteral, Token![,]>, Span),
    Variable(proc_macro2::Ident, Span),
    Null(Span),
}

impl Parse for ArgumentLiteral {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::token::Brace) {
            let span = input.span();
            let content;
            syn::braced!(content in input);

            // TODO: ideally return something other than punctuated, but need to retain spans.
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
        } else if lookahead.peek(Ident::peek_any) {
            input.parse::<Token![$]>()?;
            let ident = input.call(Ident::parse_any)?;

            // TODO: Could be true, false, null or an error?
            todo!()
        } else if lookahead.peek(syn::Lit) {
            Ok(ArgumentLiteral::Literal(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

/*
impl quote::ToTokens for FieldArgument {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let argument_name = &self.argument_name;
        let expr = &self.expr;

        tokens.append_all(quote! {
            #argument_name: #expr
        });
    }
} */

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
        assert_matches!(arguments[0].value, ArgumentLiteral::Literal(_));
    }

    #[test]
    fn test_parsing_multiple_arg_expressions() {
        let parsed: CynicArguments = parse_quote! { x: 1, y: $variable };

        let arguments = parsed.arguments.iter().collect::<Vec<_>>();

        assert_eq!(arguments.len(), 2);
        assert_eq!(arguments[0].argument_name.to_string(), "x".to_string());
        assert_matches!(arguments[0].value, ArgumentLiteral::Literal(_));

        assert_eq!(arguments[1].argument_name.to_string(), "y".to_string());
        assert_matches!(&arguments[1].value, ArgumentLiteral::Variable(name ,_) => {
            assert_eq!(name.to_string(), "variable");
        });
    }

    #[test]
    fn test_parsing_list_and_object() {
        let parsed: CynicArguments = parse_quote! { x: {fieldOne: ["hello"], fieldTwo: "hello"}};
        let arguments = parsed.arguments.iter().collect::<Vec<_>>();

        assert_eq!(arguments.len(), 1);
        assert_eq!(arguments[0].argument_name.to_string(), "x".to_string());
        assert_matches!(&arguments[0].value, ArgumentLiteral::Object(fields, _) => {
            let fields = fields.iter().collect::<Vec<_>>();
            assert_eq!(fields.len(), 2);

            assert_eq!(fields[0].argument_name.to_string(), "fieldOne");
            assert_matches!(&fields[0].value, ArgumentLiteral::List(vals, _) => {
                let vals = vals.iter().collect::<Vec<_>>();
                assert_eq!(vals.len(), 1);

                assert_matches!(vals[0], ArgumentLiteral::Literal(_));
            });

            assert_eq!(fields[1].argument_name.to_string(), "fieldTwo");
            assert_matches!(fields[1].value, ArgumentLiteral::Literal(_));
        });
    }
}
