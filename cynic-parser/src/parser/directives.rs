use super::{
    arguments::{arguments, Context},
    Parser,
};
use crate::{lexer::Token, syntax::SyntaxKind::*};

pub(super) fn maybe_directives(parser: &mut Parser, context: Context) {
    parser.skip_ws();
    if parser.current() != Some(Token::At) {
        return;
    }

    parser.builder.start_node(DIRECTIVES.into());

    while parser.current() == Some(Token::At) {
        directive(parser, context);
        parser.skip_ws();
    }

    parser.builder.finish_node();
}

fn directive(parser: &mut Parser, context: Context) {
    assert_eq!(parser.current(), Some(Token::At));
    parser.builder.start_node(DIRECTIVE.into());
    parser.bump();
    parser.skip_ws();

    match parser.current() {
        Some(Token::Name) => {
            parser.bump();
            parser.skip_ws();
        }
        _ => parser.error("expected name"),
    }

    if let Some(Token::OpenParen) = parser.current() {
        arguments(parser, context)
    }

    parser.builder.finish_node();
}
