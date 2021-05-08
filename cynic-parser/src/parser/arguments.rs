use super::{named_type, selection_set, Parser};
use crate::{lexer::Token, syntax::SyntaxKind::*};

pub(super) fn arguments(parser: &mut Parser) {
    assert_eq!(parser.current(), Some(Token::OpenParen));
    parser.builder.start_node(ARGUMENTS.into());
    parser.bump();
    // TODO: almost wonder if bump should skip ws automatically...
    parser.skip_ws();

    loop {
        match parser.current() {
            None => {
                parser.error("expected arguments");
                break;
            }
            Some(Token::Name) => {
                argument(parser);
            }
            Some(Token::CloseParen) => {
                parser.bump();
                parser.skip_ws();
                break;
            }
            _ => {
                // TODO: Is this good?  no idea
                parser.error("expected a name or )");
                break;
            }
        }
    }

    parser.builder.finish_node();
}

fn argument(parser: &mut Parser) {
    assert_eq!(parser.current(), Some(Token::Name));
    parser.builder.start_node(ARGUMENT.into());
    parser.bump();
    parser.skip_ws();

    match parser.current() {
        Some(Token::Colon) => {
            parser.bump();
        }
        _ => {
            parser.error("expected colon");
        }
    }

    value(parser);
    parser.skip_ws();
    parser.builder.finish_node();
}

fn value(parser: &mut Parser) {
    // TODO: Insert a value node here
    parser.builder.start_node(VALUE.into());
    parser.skip_ws();
    match parser.current() {
        Some(Token::Dollar) => super::variable(parser),
        Some(Token::NegativeSign) | Some(Token::Number) => number(parser),
        Some(Token::Quote) | Some(Token::BlockQuote) => string(parser),
        Some(Token::Name) => {
            match parser.current_str().unwrap() {
                "true" | "false" => boolean(parser),
                "null" => null(parser),
                other => enum_value(parser),
            }
            // Could be boolean, null, or an enum
        }
        Some(Token::OpenCurly) => object(parser),
        Some(Token::OpenSquare) => list(parser),
        _ => parser.error("expected a value"),
    }
    parser.skip_ws();
    parser.builder.finish_node();
}

fn number(parser: &mut Parser) {
    assert!(matches!(
        parser.current(),
        Some(Token::Number) | Some(Token::NegativeSign)
    ));
    let checkpoint = parser.builder.checkpoint();

    if let Some(Token::NegativeSign) = parser.current() {
        parser.bump();
    }

    match parser.current() {
        Some(Token::Number) => {
            parser.bump();
        }
        _ => parser.error("expected number"),
    }

    if let Some(Token::ExponentPart) | Some(Token::FractionalPart) = parser.current() {
        parser.builder.start_node_at(checkpoint, FLOAT_VALUE.into());
        if let Some(Token::FractionalPart) = parser.current() {
            parser.bump();
        }
        if let Some(Token::ExponentPart) = parser.current() {
            parser.bump();
        }
    } else {
        parser
            .builder
            .start_node_at(checkpoint, INTEGER_VALUE.into());
    }

    parser.builder.finish_node();
}

fn string(parser: &mut Parser) {
    assert!(matches!(
        parser.current(),
        Some(Token::Quote) | Some(Token::BlockQuote)
    ));
    // TODO: probably want to be able to differentiate between normal & block strings.
    parser.builder.start_node(STRING_VALUE.into());

    if parser.current() == Some(Token::Quote) {
        parser.bump();
        loop {
            match dbg!(parser.current()) {
                None => {
                    parser.error("unexpected end of input while parsing string");
                    break;
                }
                Some(Token::Quote) => {
                    parser.bump();
                    break;
                }
                Some(Token::LineTerminator) => {
                    parser.error("missing \"");
                    break;
                }
                _ => {
                    parser.bump_as(STRING_CONTENTS);
                }
            }
        }
    } else {
        parser.bump();
        loop {
            // TODO: I'm pretty sure this doesn't treat escape characters
            // correctly (there's also some mention of escaped block quotes that
            // I don't quite understand - figure this out later.)
            match parser.current() {
                None => {
                    parser.error("unexpected end of input while parsing string");
                    break;
                }
                Some(Token::BlockQuote) => {
                    parser.bump();
                    break;
                }
                _ => {
                    parser.bump_as(STRING_CONTENTS);
                }
            }
        }
    }

    parser.builder.finish_node();
}

fn object(parser: &mut Parser) {
    assert_eq!(parser.current(), Some(Token::OpenCurly));
    parser.builder.start_node(OBJECT_VALUE.into());
    parser.bump();
    parser.skip_ws();

    loop {
        match parser.current() {
            None => {
                parser.error("missing }");
            }
            Some(Token::CloseCurly) => {
                parser.bump();
                break;
            }
            Some(Token::Name) => {
                object_field(parser);
            }
            _ => {
                parser.error("expected name");
                break;
            }
        }
    }

    parser.builder.finish_node();
}

fn object_field(parser: &mut Parser) {
    assert_eq!(parser.current(), Some(Token::Name));
    parser.builder.start_node(OBJECT_FIELD.into());
    parser.bump();
    parser.skip_ws();

    match parser.current() {
        Some(Token::Colon) => {
            parser.bump();
        }
        _ => parser.error("expected colon"),
    }
    value(parser);

    parser.builder.finish_node();
}

fn list(parser: &mut Parser) {
    assert_eq!(parser.current(), Some(Token::OpenSquare));
    parser.builder.start_node(LIST_VALUE.into());
    parser.bump();
    parser.skip_ws();
    loop {
        match parser.current() {
            None => {
                parser.error("missing ]");
                break;
            }
            Some(Token::CloseSquare) => {
                parser.bump();
                break;
            }
            _ => {}
        }
        value(parser);
        parser.skip_ws();
    }

    parser.builder.finish_node();
}

fn boolean(parser: &mut Parser) {
    assert_eq!(parser.current(), Some(Token::Name));

    match parser.current_str() {
        Some("true") => parser.bump_as(TRUE),
        Some("false") => parser.bump_as(FALSE),
        _ => panic!("boolean pre-condition not met"),
    }
}

fn null(parser: &mut Parser) {
    assert_eq!(parser.current(), Some(Token::Name));
    assert_eq!(parser.current_str(), Some("null"));

    parser.bump_as(NULL);
}

fn enum_value(parser: &mut Parser) {
    assert_eq!(parser.current(), Some(Token::Name));

    parser.builder.start_node(ENUM_VALUE.into());
    parser.bump();
    parser.builder.finish_node();
}
