mod tokens;

use std::fmt;

use logos::{Logos, SpannedIter};
pub use tokens::*;

use crate::{parser::AdditionalErrors, Span};

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexicalError {
    InvalidToken(String, Span),
}

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

impl LexicalError {
    pub fn span(&self) -> Span {
        match self {
            LexicalError::InvalidToken(_, span) => *span,
        }
    }
}

impl fmt::Display for LexicalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexicalError::InvalidToken(token, _span) => write!(f, "invalid token: {token}"),
        }
    }
}

impl std::error::Error for LexicalError {}
