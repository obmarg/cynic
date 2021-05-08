use logos::Logos;

#[derive(Logos, Debug, PartialEq)]
enum Token {
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
    DecimalPart,

    // Literal values
    #[token("true")]
    True,
    #[token("false")]
    False,

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

    // Logos requires one token variant to handle errors,
    // it can be named anything you wish.
    #[error]
    Other,
}

// TODO: Consider using ungrammar for the CST side of things?

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_lex_ints() {
        assert_eq!(Token::lexer("1").collect::<Vec<_>>(), vec![Token::Number]);
        assert_eq!(
            Token::lexer("-1").collect::<Vec<_>>(),
            vec![Token::NegativeSign, Token::Number]
        );
        assert_eq!(
            Token::lexer("-0").collect::<Vec<_>>(),
            vec![Token::NegativeSign, Token::Number]
        );
        assert_eq!(Token::lexer("100").collect::<Vec<_>>(), vec![Token::Number]);
        assert_eq!(
            Token::lexer("-100").collect::<Vec<_>>(),
            vec![Token::NegativeSign, Token::Number]
        );
    }

    #[test]
    fn can_lex_graphql_queries() {
        let lexer = Token::lexer(
            r#"
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

        "#,
        );

        insta::assert_debug_snapshot!(lexer.collect::<Vec<_>>());
    }
}
