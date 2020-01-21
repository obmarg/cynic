use proc_macro2::TokenStream;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Expr, Ident, Result, Token,
};

pub fn arguments_from_field_attrs(attrs: &Vec<syn::Attribute>) -> Result<Vec<FieldArgument>> {
    for attr in attrs {
        if attr.path.is_ident("cynic_arguments") {
            let parsed: CynicArguments = attr.parse_args()?;
            return Ok(parsed.arguments.into_iter().collect());
        }
    }
    Ok(vec![])
}

/// Implements syn::Parse to parse out arguments from the cynic_arguments
/// attribute.
#[derive(PartialEq, Debug)]
struct CynicArguments {
    arguments: Punctuated<FieldArgument, Token![,]>,
}

impl Parse for CynicArguments {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(CynicArguments {
            arguments: Punctuated::parse_terminated(input)?,
        })
    }
}

#[derive(PartialEq, Debug)]
pub enum ArgumentExpression {
    Literal(syn::Lit),
    FieldAccess(syn::ExprField),
}

impl std::convert::TryFrom<Expr> for ArgumentExpression {
    type Error = syn::Error;

    fn try_from(expr: Expr) -> Result<ArgumentExpression> {
        use syn::spanned::Spanned;

        match expr {
            Expr::Lit(expr) => Ok(ArgumentExpression::Literal(expr.lit)),
            Expr::Field(field_expr) => {
                if let Expr::Path(expr) = *field_expr.base.clone() {
                    if expr.path.is_ident("args") {
                        Ok(ArgumentExpression::FieldAccess(field_expr))
                    } else {
                        Err(syn::Error::new(
                            expr.span(),
                            "Query arguments are exposed on a struct named args",
                        ))
                    }
                } else {
                    Err(syn::Error::new(
                        field_expr.base.span(),
                        "Arguments should be of the form args.an_argument",
                    ))
                }
            }
            _ => Err(syn::Error::new(
                expr.span(),
                format!("Must be a literal or an expression of the form args.an_argument"),
            )),
        }
    }
}

impl quote::ToTokens for ArgumentExpression {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let result = match self {
            ArgumentExpression::Literal(expr) => quote! { #expr },
            ArgumentExpression::FieldAccess(expr) => quote! { #expr },
        };

        tokens.append_all(result);
    }
}

#[derive(PartialEq, Debug)]
pub struct FieldArgument {
    pub argument_name: Ident,
    //expr: ArgumentExpression,
    // TODO: Remove argument expression maybe?  seems too much work to be worth it just now.
    pub expr: syn::Expr,
}

impl Parse for FieldArgument {
    fn parse(input: ParseStream) -> Result<Self> {
        use std::convert::TryInto;

        let argument_name = input.parse()?;
        input.parse::<Token![=]>()?;
        let expr: Expr = input.parse()?;

        Ok(FieldArgument {
            argument_name,
            expr, //expr.try_into()?,
        })
    }
}

impl quote::ToTokens for FieldArgument {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let argument_name = &self.argument_name;
        let expr = &self.expr;

        tokens.append_all(quote! {
            #argument_name: #expr
        });
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use assert_matches::assert_matches;
    use syn::parse_quote;

    #[test]
    fn test_parsing_string_literal() {
        let parsed: CynicArguments = parse_quote! { x = "abcd" };

        let arguments = parsed.arguments.iter().collect::<Vec<_>>();

        assert_eq!(arguments.len(), 1);
        assert_eq!(arguments[0].argument_name.to_string(), "x".to_string());
    }

    #[test]
    fn test_parsing_multiple_arg_expressions() {
        let parsed: CynicArguments = parse_quote! { x = 1, y = args.test };

        let arguments = parsed.arguments.iter().collect::<Vec<_>>();

        assert_eq!(arguments.len(), 2);
        assert_eq!(arguments[0].argument_name.to_string(), "x".to_string());
        /*
        TODO: Re-instate

        assert_eq!(
            arguments[0].expr,
            ArgumentExpression::Literal(parse_quote! { 1 })
        );
        assert_eq!(arguments[1].argument_name.to_string(), "y".to_string());
        assert_matches!(arguments[1].expr, ArgumentExpression::FieldAccess(_));
        */
    }
}
