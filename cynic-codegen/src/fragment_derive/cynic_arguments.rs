use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Expr, Ident, Result, Token,
};

/// Implements syn::Parse to parse out arguments from the cynic_arguments
/// attribute.
#[derive(PartialEq, Debug)]
struct CynicArguments {
    arguments: Punctuated<Argument, Token![,]>,
}

impl Parse for CynicArguments {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(CynicArguments {
            arguments: Punctuated::parse_terminated(input)?,
        })
    }
}

#[derive(PartialEq, Debug)]
struct Argument {
    argument_name: Ident,
    equals: Token![=],
    expr: Expr,
}

impl Parse for Argument {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Argument {
            argument_name: input.parse()?,
            equals: input.parse()?,
            expr: input.parse()?,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_parsing_string_literal() {
        use syn::ExprLit;

        let parsed: CynicArguments = parse_quote! { x = "abcd" };

        let arguments = parsed.arguments.iter().collect::<Vec<_>>();

        assert_eq!(arguments.len(), 1);
        assert_eq!(arguments[0].argument_name.to_string(), "x".to_string());
    }

    #[test]
    fn test_parsing_multiple_arg_expressions() {
        use syn::ExprLit;

        let parsed: CynicArguments = parse_quote! { x = 1, y = args.test() };

        let arguments = parsed.arguments.iter().collect::<Vec<_>>();

        assert_eq!(arguments.len(), 2);
        assert_eq!(arguments[0].argument_name.to_string(), "x".to_string());
        assert_eq!(arguments[1].argument_name.to_string(), "y".to_string());
    }
}
