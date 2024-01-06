/*
* The code in this file is taken from
* https://github.com/facebook/relay/blob/main/compiler/crates/graphql-syntax/src/lexer.rs
*
* Licensed under the MIT license:
*
* Copyright (c) Meta Platforms, Inc. and affiliates.
*
* Permission is hereby granted, free of charge, to any person obtaining a copy
* of this software and associated documentation files (the "Software"), to deal
* in the Software without restriction, including without limitation the rights
* to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
* copies of the Software, and to permit persons to whom the Software is
* furnished to do so, subject to the following conditions:
*
* The above copyright notice and this permission notice shall be included in all
* copies or substantial portions of the Software.
*
* THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
* IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
* FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
* AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
* LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
* OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
* SOFTWARE.
*/

use std::fmt;

use logos::Lexer;
use logos::Logos;

#[derive(Default, Eq, PartialEq)]
pub struct TokenExtras {
    /// Token callbacks might store an error token kind in here before failing.
    /// This is then picked up in the parser to turn the `Error` token into a
    /// more specific variant.
    pub error_token: Option<Token<'static>>,
}

/// Lexer for the GraphQL specification: http://spec.graphql.org/
#[derive(Logos, Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[logos(extras = TokenExtras, skip r"[ \t\r\n\f,\ufeff]+|#[^\n\r]*")]
pub enum Token<'a> {
    ErrorUnterminatedString,
    ErrorUnsupportedStringCharacter,
    ErrorUnterminatedBlockString,
    Empty,

    // Valid tokens
    #[token("&")]
    Ampersand,

    #[token("@")]
    At,

    #[token("}")]
    CloseBrace,

    #[token("]")]
    CloseBracket,

    #[token(")")]
    CloseParen,

    #[token(":")]
    Colon,

    #[token("$")]
    Dollar,

    EndOfFile,

    #[token("=")]
    Equals,

    #[token("!")]
    Exclamation,

    #[token("schema")]
    Schema,

    #[token("query")]
    Query,

    #[token("type")]
    Type,

    // IntegerPart:    -?(0|[1-9][0-9]*)
    // FractionalPart: \\.[0-9]+
    // ExponentPart:   [eE][+-]?[0-9]+
    #[regex("-?(0|[1-9][0-9]*)(\\.[0-9]+[eE][+-]?[0-9]+|\\.[0-9]+|[eE][+-]?[0-9]+)")]
    FloatLiteral,

    #[regex("[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice())]
    Identifier(&'a str),

    #[regex("-?(0|[1-9][0-9]*)")]
    IntegerLiteral,

    #[regex("-?0[0-9]+(\\.[0-9]+[eE][+-]?[0-9]+|\\.[0-9]+|[eE][+-]?[0-9]+)?")]
    ErrorNumberLiteralLeadingZero,

    #[regex("-?(0|[1-9][0-9]*)(\\.[0-9]+[eE][+-]?[0-9]+|\\.[0-9]+|[eE][+-]?[0-9]+)?[.a-zA-Z_]")]
    ErrorNumberLiteralTrailingInvalid,

    #[regex("-?(\\.[0-9]+[eE][+-]?[0-9]+|\\.[0-9]+)")]
    ErrorFloatLiteralMissingZero,

    #[token("{")]
    OpenBrace,

    #[token("[")]
    OpenBracket,

    #[token("(")]
    OpenParen,

    #[token(".")]
    Period,

    #[token("..")]
    PeriodPeriod,

    #[token("|")]
    Pipe,

    #[token("...")]
    Spread,

    #[token("\"", lex_string)]
    StringLiteral,

    #[token("\"\"\"", lex_block_string)]
    BlockStringLiteral,
}

#[derive(Logos, Debug)]
pub enum StringToken {
    #[regex(r#"\\["\\/bfnrt]"#)]
    EscapedCharacter,

    #[regex(r#"\\u[0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f][0-9A-Fa-f]"#)]
    EscapedUnicode,

    #[token("\"")]
    Quote,

    #[regex(r#"\n|\r|\r\n"#)]
    LineTerminator,

    #[regex(r#"[\u0009\u0020\u0021\u0023-\u005B\u005D-\uFFFF]+"#)]
    StringCharacters,
}

fn lex_string<'a>(lexer: &mut Lexer<'a, Token<'a>>) -> bool {
    let remainder = lexer.remainder();
    let mut string_lexer = StringToken::lexer(remainder);
    while let Some(string_token) = string_lexer.next() {
        match string_token {
            Ok(StringToken::Quote) => {
                lexer.bump(string_lexer.span().end);
                return true;
            }
            Ok(StringToken::LineTerminator) => {
                lexer.bump(string_lexer.span().start);
                lexer.extras.error_token = Some(Token::ErrorUnterminatedString);
                return false;
            }
            Ok(
                StringToken::EscapedCharacter
                | StringToken::EscapedUnicode
                | StringToken::StringCharacters,
            ) => {}
            Err(_) => {
                lexer.extras.error_token = Some(Token::ErrorUnsupportedStringCharacter);
                return false;
            }
        }
    }
    lexer.extras.error_token = Some(Token::ErrorUnterminatedString);
    false
}

fn lex_block_string<'a>(lexer: &mut Lexer<'a, Token<'a>>) -> bool {
    let remainder = lexer.remainder();
    let mut string_lexer = BlockStringToken::lexer(remainder);
    while let Some(string_token) = string_lexer.next() {
        match string_token {
            Ok(BlockStringToken::TripleQuote) => {
                lexer.bump(string_lexer.span().end);
                return true;
            }
            Ok(BlockStringToken::EscapedTripleQuote | BlockStringToken::Other) => {}
            Err(_) => unreachable!(),
        }
    }
    lexer.extras.error_token = Some(Token::ErrorUnterminatedBlockString);
    false
}

#[derive(Logos, Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum BlockStringToken {
    #[token("\\\"\"\"")]
    EscapedTripleQuote,

    #[token("\"\"\"")]
    TripleQuote,

    #[regex(r#"[\u0009\u000A\u000D\u0020-\uFFFF]"#)]
    Other,
}

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Token::Ampersand => "ampersand ('&')",
            Token::At => "at ('@')",
            Token::CloseBrace => "closing brace ('}')",
            Token::CloseBracket => "closing bracket (']')",
            Token::CloseParen => "closing paren (')')",
            Token::Colon => "colon (':')",
            Token::Dollar => "dollar ('$')",
            Token::EndOfFile => "end of file",
            Token::Equals => "equals ('=')",
            Token::Exclamation => "exclamation mark ('!')",
            Token::FloatLiteral => "floating point value (e.g. '3.14')",
            Token::Identifier(_) => "non-variable identifier (e.g. 'x' or 'Foo')",
            Token::IntegerLiteral => "integer value (e.g. '0' or '42')",
            Token::OpenBrace => "open brace ('{')",
            Token::OpenBracket => "open bracket ('[')",
            Token::OpenParen => "open parenthesis ('(')",
            Token::Period => "period ('.')",
            Token::PeriodPeriod => "double period ('..')",
            Token::Pipe => "pipe ('|')",
            Token::Spread => "spread ('...')",
            Token::BlockStringLiteral => "block string (e.g. '\"\"\"hi\"\"\"')",
            Token::ErrorFloatLiteralMissingZero => "unsupported number (int or float) literal",
            Token::ErrorNumberLiteralLeadingZero => "unsupported number (int or float) literal",
            Token::ErrorNumberLiteralTrailingInvalid => "unsupported number (int or float) literal",
            Token::StringLiteral => "string literal (e.g. '\"...\"')",
            Token::ErrorUnterminatedString => "unterminated string",
            Token::ErrorUnsupportedStringCharacter => "unsupported character in string",
            Token::ErrorUnterminatedBlockString => "unterminated block string",
            Token::Empty => "missing expected kind",
            Token::Schema => "schema",
            Token::Query => "query",
            Token::Type => "type",
        };
        f.write_str(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_token(source: &str, kind: Token, length: usize) {
        let mut lexer = Token::lexer(source);
        assert_eq!(
            lexer.next(),
            Some(Ok(kind)),
            "Testing the lexing of string '{}'",
            source
        );
        assert_eq!(
            lexer.span(),
            0..length,
            "Testing the lexing of string '{}'",
            source
        );
    }

    fn assert_error(source: &str, length: usize) {
        let mut lexer = Token::lexer(source);
        assert_eq!(
            lexer.next(),
            Some(Err(())),
            "Testing lexing fails for string '{}'",
            source
        );
        assert_eq!(
            lexer.span(),
            0..length,
            "Testing the lexing of string '{}'",
            source
        );
    }

    #[test]
    fn test_number_successes() {
        assert_token("4", Token::IntegerLiteral, 1);
        assert_token("4.123", Token::FloatLiteral, 5);
        assert_token("-4", Token::IntegerLiteral, 2);
        assert_token("9", Token::IntegerLiteral, 1);
        assert_token("0", Token::IntegerLiteral, 1);
        assert_token("-4.123", Token::FloatLiteral, 6);
        assert_token("0.123", Token::FloatLiteral, 5);
        assert_token("123e4", Token::FloatLiteral, 5);
        assert_token("123E4", Token::FloatLiteral, 5);
        assert_token("123e-4", Token::FloatLiteral, 6);
        assert_token("123e+4", Token::FloatLiteral, 6);
        assert_token("-1.123e4", Token::FloatLiteral, 8);
        assert_token("-1.123E4", Token::FloatLiteral, 8);
        assert_token("-1.123e-4", Token::FloatLiteral, 9);
        assert_token("-1.123e+4", Token::FloatLiteral, 9);
        assert_token("-1.123e4567", Token::FloatLiteral, 11);
        assert_token("-0", Token::IntegerLiteral, 2);
    }

    #[test]
    fn test_number_failures() {
        assert_token("00", Token::ErrorNumberLiteralLeadingZero, 2);
        assert_token("01", Token::ErrorNumberLiteralLeadingZero, 2);
        assert_token("-01", Token::ErrorNumberLiteralLeadingZero, 3);
        assert_error("+1", 1);
        assert_token("01.23", Token::ErrorNumberLiteralLeadingZero, 5);
        assert_token("1.", Token::ErrorNumberLiteralTrailingInvalid, 2);
        assert_token("1e", Token::ErrorNumberLiteralTrailingInvalid, 2);
        assert_token("1.e1", Token::ErrorNumberLiteralTrailingInvalid, 2);
        assert_token("1.A", Token::ErrorNumberLiteralTrailingInvalid, 2);
        assert_error("-A", 1);
        assert_token("1.0e", Token::ErrorNumberLiteralTrailingInvalid, 4);
        assert_token("1.0eA", Token::ErrorNumberLiteralTrailingInvalid, 4);
        assert_token("1.2e3e", Token::ErrorNumberLiteralTrailingInvalid, 6);
        assert_token("1.2e3.4", Token::ErrorNumberLiteralTrailingInvalid, 6);
        assert_token("1.23.4", Token::ErrorNumberLiteralTrailingInvalid, 5);
        assert_token(".123", Token::ErrorFloatLiteralMissingZero, 4);

        // check that we don't consume trailing valid items
        assert_token("1.23.{}", Token::ErrorNumberLiteralTrailingInvalid, 5);
        assert_token("1.23. {}", Token::ErrorNumberLiteralTrailingInvalid, 5);
        assert_token("1.23. []", Token::ErrorNumberLiteralTrailingInvalid, 5);
        assert_token("1.23. foo", Token::ErrorNumberLiteralTrailingInvalid, 5);
        assert_token("1.23. $foo", Token::ErrorNumberLiteralTrailingInvalid, 5);
    }

    #[test]
    fn test_lexing() {
        let input = "
           query EmptyQuery($id: ID!) {
             node(id: $id) {
               id @skip(if: false)
               ...E1
             }
           }
         ";
        let mut lexer = Token::lexer(input);

        assert_eq!(lexer.next(), Some(Ok(Token::Identifier("query"))));
        assert_eq!(lexer.slice(), "query");

        assert_eq!(lexer.next(), Some(Ok(Token::Identifier("EmptyQuery"))));
        assert_eq!(lexer.slice(), "EmptyQuery");

        assert_eq!(lexer.next(), Some(Ok(Token::OpenParen)));
        assert_eq!(lexer.slice(), "(");

        assert_eq!(lexer.next(), Some(Ok(Token::Dollar)));
        assert_eq!(lexer.slice(), "$");

        assert_eq!(lexer.next(), Some(Ok(Token::Identifier("id"))));
        assert_eq!(lexer.slice(), "id");

        assert_eq!(lexer.next(), Some(Ok(Token::Colon)));
        assert_eq!(lexer.slice(), ":");

        assert_eq!(lexer.next(), Some(Ok(Token::Identifier("ID"))));
        assert_eq!(lexer.slice(), "ID");

        assert_eq!(lexer.next(), Some(Ok(Token::Exclamation)));
        assert_eq!(lexer.slice(), "!");

        assert_eq!(lexer.next(), Some(Ok(Token::CloseParen)));
        assert_eq!(lexer.slice(), ")");

        assert_eq!(lexer.next(), Some(Ok(Token::OpenBrace)));
        assert_eq!(lexer.slice(), "{");

        assert_eq!(lexer.next(), Some(Ok(Token::Identifier("node"))));
        assert_eq!(lexer.slice(), "node");

        assert_eq!(lexer.next(), Some(Ok(Token::OpenParen)));
        assert_eq!(lexer.slice(), "(");

        assert_eq!(lexer.next(), Some(Ok(Token::Identifier("id"))));
        assert_eq!(lexer.slice(), "id");

        assert_eq!(lexer.next(), Some(Ok(Token::Colon)));
        assert_eq!(lexer.slice(), ":");

        assert_eq!(lexer.next(), Some(Ok(Token::Dollar)));
        assert_eq!(lexer.slice(), "$");

        assert_eq!(lexer.next(), Some(Ok(Token::Identifier("id"))));
        assert_eq!(lexer.slice(), "id");

        assert_eq!(lexer.next(), Some(Ok(Token::CloseParen)));
        assert_eq!(lexer.slice(), ")");

        assert_eq!(lexer.next(), Some(Ok(Token::OpenBrace)));
        assert_eq!(lexer.slice(), "{");

        assert_eq!(lexer.next(), Some(Ok(Token::Identifier("id"))));
        assert_eq!(lexer.slice(), "id");

        assert_eq!(lexer.next(), Some(Ok(Token::At)));
        assert_eq!(lexer.slice(), "@");

        assert_eq!(lexer.next(), Some(Ok(Token::Identifier("skip"))));
        assert_eq!(lexer.slice(), "skip");

        assert_eq!(lexer.next(), Some(Ok(Token::OpenParen)));
        assert_eq!(lexer.slice(), "(");

        assert_eq!(lexer.next(), Some(Ok(Token::Identifier("if"))));
        assert_eq!(lexer.slice(), "if");

        assert_eq!(lexer.next(), Some(Ok(Token::Colon)));
        assert_eq!(lexer.slice(), ":");

        assert_eq!(lexer.next(), Some(Ok(Token::Identifier("false"))));
        assert_eq!(lexer.slice(), "false");

        assert_eq!(lexer.next(), Some(Ok(Token::CloseParen)));
        assert_eq!(lexer.slice(), ")");

        assert_eq!(lexer.next(), Some(Ok(Token::Spread)));
        assert_eq!(lexer.slice(), "...");

        assert_eq!(lexer.next(), Some(Ok(Token::Identifier("E1"))));
        assert_eq!(lexer.slice(), "E1");

        assert_eq!(lexer.next(), Some(Ok(Token::CloseBrace)));
        assert_eq!(lexer.slice(), "}");

        assert_eq!(lexer.next(), Some(Ok(Token::CloseBrace)));
        assert_eq!(lexer.slice(), "}");

        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn test_string_lexing() {
        let input = r#"
             "test"
             "escaped \" quote"
             "unterminated
             "
         "#;
        let mut lexer = Token::lexer(input);

        assert_eq!(lexer.next(), Some(Ok(Token::StringLiteral)));
        assert_eq!(lexer.slice(), "\"test\"");

        assert_eq!(lexer.next(), Some(Ok(Token::StringLiteral)));
        assert_eq!(lexer.slice(), r#""escaped \" quote""#);

        assert_eq!(lexer.next(), Some(Err(())));
        assert_eq!(
            lexer.extras.error_token,
            Some(Token::ErrorUnterminatedString)
        );
        assert_eq!(lexer.slice(), "\"unterminated");
    }

    #[test]
    fn test_invalid_character_lexing() {
        let input = r#"
             {
                 %%%
                 __typename
                 *
             }
         "#;
        let mut lexer = Token::lexer(input);

        assert_eq!(lexer.next(), Some(Ok(Token::OpenBrace)));
        assert_eq!(lexer.slice(), "{");

        assert_eq!(lexer.next(), Some(Err(())));
        assert_eq!(lexer.slice(), "%");

        assert_eq!(lexer.next(), Some(Err(())));
        assert_eq!(lexer.slice(), "%");

        assert_eq!(lexer.next(), Some(Err(())));
        assert_eq!(lexer.slice(), "%");

        assert_eq!(lexer.next(), Some(Ok(Token::Identifier("__typename"))));
        assert_eq!(lexer.slice(), "__typename");

        assert_eq!(lexer.next(), Some(Err(())));
        assert_eq!(lexer.slice(), "*");

        assert_eq!(lexer.next(), Some(Ok(Token::CloseBrace)));
        assert_eq!(lexer.slice(), "}");

        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn test_block_string_lexing() {
        let input = r#"
             # escaped
             """tes\"""t"""
             # empty
             """"""
             # 2 quotes in a string
             """"" """
             """
                 multi-
                 line
             """
             """unterminated
         "#;
        let mut lexer = Token::lexer(input);

        assert_eq!(lexer.next(), Some(Ok(Token::BlockStringLiteral)));
        assert_eq!(lexer.slice(), r#""""tes\"""t""""#);

        assert_eq!(lexer.next(), Some(Ok(Token::BlockStringLiteral)));
        assert_eq!(lexer.slice(), r#""""""""#);

        assert_eq!(lexer.next(), Some(Ok(Token::BlockStringLiteral)));
        assert_eq!(lexer.slice(), r#"""""" """"#);

        assert_eq!(lexer.next(), Some(Ok(Token::BlockStringLiteral)));
        assert_eq!(
            lexer.slice(),
            r#""""
                 multi-
                 line
             """"#
        );

        assert_eq!(lexer.next(), Some(Err(())));
        assert_eq!(
            lexer.extras.error_token,
            Some(Token::ErrorUnterminatedBlockString)
        );
        // Unterminated string just consumes the starting quotes
        assert_eq!(lexer.slice(), r#"""""#);
    }

    #[test]
    fn test_bom_lexing() {
        let input = "\u{feff}";

        let mut lexer = Token::lexer(input);

        assert_eq!(lexer.next(), None);
    }
}
