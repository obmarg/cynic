use logos::{Logos, SpannedIter};

#[derive(Logos, Debug, PartialEq, Clone, Copy)]
pub enum Token<'a> {
    #[token(",")]
    Comma,

    #[token("#")]
    Hash,

    // Punctuators
    #[token("!")]
    Bang,
    #[token("$")]
    Dollar,
    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,
    #[token("...")]
    Spread,
    #[token(":")]
    Colon,
    #[token("=")]
    Equals,
    #[token("@")]
    At,
    #[token("[")]
    OpenSquare,
    #[token("]")]
    CloseSquare,
    #[token("{")]
    OpenCurly,
    #[token("}")]
    CloseCurly,
    #[token("|")]
    Pipe,

    // TODO: These might be temporary, not sure
    #[token("schema")]
    Schema,
    #[token("query")]
    Query,
    #[token("type")]
    Type,

    #[regex(r"[_A-Za-z][_0-9A-Za-z]*", |lex| lex.slice())]
    Name(&'a str),

    // Numbers
    #[token("-")]
    NegativeSign,
    #[regex(r"(0)|([1-9][0-9]*)")]
    Number,
    #[regex(r"[eE][+-]?[0-9]+")]
    ExponentPart,
    #[regex(r"\.[0-9]+")]
    FractionalPart,

    #[token("\"")]
    Quote,

    #[token("\"\"\"")]
    BlockQuote,

    #[regex(r"[ \t\f]+")]
    Whitespace,

    #[regex(r"\\u[0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f]/")]
    EscapedUnicode,

    #[regex(r#"\\["\\/bfnrt]"#)]
    EscapedCharacter,

    #[regex(r"\n|(\r[^\n])|\r\n")]
    LineTerminator,
}

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
        loop {
            match self.token_stream.next() {
                None => return None,
                Some((Ok(Token::Whitespace | Token::Comma), _)) => continue,
                Some((Ok(token), span)) => return Some(Ok((span.start, token, span.end))),
                Some((Err(_), _)) => return Some(Err(LexicalError::InvalidToken)),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_lex_ints() {
        assert_eq!(lex("1"), vec![Token::Number]);
        assert_eq!(lex("-1"), vec![Token::NegativeSign, Token::Number]);
        assert_eq!(lex("-0"), vec![Token::NegativeSign, Token::Number]);
        assert_eq!(lex("100"), vec![Token::Number]);
        assert_eq!(lex("-100"), vec![Token::NegativeSign, Token::Number]);
    }

    #[test]
    fn can_lex_variables() {
        let res = lex(r#"$name: String! = "hello""#);
        assert_eq!(
            res,
            vec![
                Token::Dollar,
                Token::Name("name"),
                Token::Colon,
                Token::Whitespace,
                Token::Name("String"),
                Token::Bang,
                Token::Whitespace,
                Token::Equals,
                Token::Whitespace,
                // TODO: this is a bad tokenization, i should work on that.
                Token::Quote,
                Token::Name("hello"),
                Token::Quote
            ]
        );
    }

    #[test]
    fn can_lex_graphql_queries() {
        let tokens = lex(r#"
        query {
            repository(owner: "obmarg", name: "cynic") {
              issueOrPullRequest(number: 1) {
                ... on Issue {
                  id
                  title
                  lastEditedAt
                }
                ... on PullRequest {
                  id
                  title
                  lastEditedAt
                }
              }
            }
          }

        "#);

        insta::assert_debug_snapshot!(tokens);
    }

    fn lex(input: &str) -> Vec<Token> {
        Token::lexer(input).map(|result| result.unwrap()).collect()
    }
}
