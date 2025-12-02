use std::fmt;

use logos::{Logos, SpannedIter};

use crate::{
    Span,
    lexer::{LexicalError, Spanned, TokenExtras},
    parser::AdditionalErrors,
};

pub struct Lexer<'input> {
    // instead of an iterator over characters, we have a token iterator
    token_stream: SpannedIter<'input, Token<'input>>,
    input: &'input str,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        Self {
            token_stream: Token::lexer(input).spanned(),
            input,
        }
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<Token<'input>, usize, AdditionalErrors>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.token_stream.next() {
            None => None,
            Some((Ok(token), span)) => Some(Ok((span.start, token, span.end))),
            Some((Err(_), span)) => {
                Some(Err(AdditionalErrors::Lexical(LexicalError::InvalidToken(
                    self.input[span.start..span.end].to_string(),
                    Span::new(span.start, span.end),
                ))))
            }
        }
    }
}

/// Lexer for GraphQL schema coordinates: https://spec.graphql.org/September2025/#sec-Schema-Coordinates.Parsing-a-Schema-Coordinate
#[derive(Logos, Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[logos(extras = TokenExtras, skip r"[ \t\r\n\f,\ufeff]+|#[^\n\r]*")]
pub enum Token<'a> {
    // Valid tokens
    #[token("@")]
    At,

    #[token(")")]
    CloseParen,

    #[token(":")]
    Colon,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice())]
    Identifier(&'a str),

    #[token("(")]
    OpenParen,

    #[token(".")]
    Dot,
}

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Token::At => "at ('@')",
            Token::CloseParen => "closing paren (')')",
            Token::Colon => "colon (':')",
            Token::Identifier(_) => "identifier",
            Token::OpenParen => "opening paren ('(')",
            Token::Dot => "dot ('.')",
        })
    }
}

impl From<lalrpop_util::ParseError<usize, Token<'_>, AdditionalErrors>> for crate::Error {
    fn from(value: lalrpop_util::ParseError<usize, Token<'_>, AdditionalErrors>) -> Self {
        use crate::Error;
        use lalrpop_util::ParseError;

        match value {
            ParseError::InvalidToken { location } => Error::InvalidToken { location },
            ParseError::UnrecognizedEof { location, expected } => {
                Error::UnrecognizedEof { location, expected }
            }
            ParseError::UnrecognizedToken {
                token: (lspan, token, rspan),
                expected,
            } => Error::UnrecognizedToken {
                token: (lspan, token.to_string(), rspan),
                expected,
            },
            ParseError::ExtraToken {
                token: (lspan, token, rspan),
            } => Error::ExtraToken {
                token: (lspan, token.to_string(), rspan),
            },
            ParseError::User {
                error: AdditionalErrors::Lexical(error),
            } => Error::Lexical(error),
            ParseError::User {
                error: AdditionalErrors::MalformedString(error),
            } => Error::MalformedStringLiteral(error),
            ParseError::User {
                error: AdditionalErrors::MalformedDirectiveLocation(lhs, location, rhs),
            } => Error::MalformedDirectiveLocation(lhs, location, rhs),
            ParseError::User {
                error: AdditionalErrors::VariableInConstPosition(lhs, name, rhs),
            } => Error::MalformedDirectiveLocation(lhs, name, rhs),
        }
    }
}
