mod tokens;

use logos::{Logos, SpannedIter};
pub use tokens::*;

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

#[derive(Debug)]
pub enum LexicalError {
    InvalidToken,
}

pub struct Lexer<'input> {
    // instead of an iterator over characters, we have a token iterator
    token_stream: SpannedIter<'input, Token<'input>>,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        Self {
            token_stream: Token::lexer(input).spanned(),
        }
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<Token<'input>, usize, LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.token_stream.next() {
            None => return None,
            Some((Ok(token), span)) => return Some(Ok((span.start, token, span.end))),
            Some((Err(_), _)) => return Some(Err(LexicalError::InvalidToken)),
        }
    }
}
