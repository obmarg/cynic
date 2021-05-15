use super::{arguments, directives::maybe_directives, named_type, selection_set, Parser};
use crate::{lexer::Token, syntax::SyntaxKind::*};

pub(super) fn fragment(parser: &mut Parser) {
    fragment_name(parser);

    type_condition(parser);

    maybe_directives(parser, arguments::Context::NonConstant);

    parser.skip_ws();
    match parser.current() {
        Some(Token::OpenCurly) => selection_set(parser),
        _ => parser.error("expected selection set"),
    }
}

pub(super) fn fragment_spread(parser: &mut Parser) {
    assert_eq!(parser.current(), Some(Token::Spread));
    parser.builder.start_node(FRAGMENT_SPREAD.into());
    parser.bump();
    parser.skip_ws();
    if let Some(Token::Name) = parser.current() {
        fragment_name(parser);
    }

    maybe_directives(parser, arguments::Context::NonConstant);

    parser.skip_ws();
    if parser.current() == Some(Token::OpenCurly) {
        super::selection_set(parser);
    }

    parser.builder.finish_node();
}

fn fragment_name(parser: &mut Parser) {
    parser.skip_ws();
    match parser.current() {
        None => parser.error("expected fragment name"),
        Some(Token::Name) if parser.current_str() != Some("on") => {
            parser.builder.start_node(FRAGMENT_NAME.into());
            parser.bump();
            parser.builder.finish_node();
        }
        _ => {
            parser.error("expected fragment name");
        }
    }
}

pub(super) fn inline_fragment(parser: &mut Parser) {
    assert_eq!(parser.current(), Some(Token::Spread));
    parser.builder.start_node(INLINE_FRAGMENT.into());
    parser.bump();
    parser.skip_ws();
    if let Some((Token::Name, "on")) = parser.current_pair() {
        type_condition(parser);
    }

    maybe_directives(parser, arguments::Context::NonConstant);

    parser.skip_ws();
    match parser.current() {
        Some(Token::OpenCurly) => selection_set(parser),
        _ => parser.error("expected selection set"),
    }

    parser.builder.finish_node();
}

fn type_condition(parser: &mut Parser) {
    parser.skip_ws();
    match parser.current_pair() {
        Some((Token::Name, "on")) => {
            parser.builder.start_node(TYPE_CONDITION.into());
            parser.bump_as(ON);
            parser.skip_ws();
            if let Some(Token::Name) = parser.current() {
                named_type(parser);
            } else {
                parser.error("expected named type");
            }
            parser.builder.finish_node();
        }
        _ => {
            parser.error("expected a type condition");
        }
    }
}
