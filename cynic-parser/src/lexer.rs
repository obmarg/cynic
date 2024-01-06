use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone, Copy)]
pub enum Token {
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
    #[token("|")]
    Pipe,
    #[token("}")]
    CloseCurly,

    #[regex(r"[_A-Za-z][_0-9A-Za-z]*")]
    Name,

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
                Token::Name,
                Token::Colon,
                Token::Whitespace,
                Token::Name,
                Token::Bang,
                Token::Whitespace,
                Token::Equals,
                Token::Whitespace,
                Token::Quote,
                Token::Name,
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
