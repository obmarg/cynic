#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[repr(u16)]
pub enum SyntaxKind {
    // Tokens:
    COMMA,
    HASH,
    BANG,
    DOLLAR,
    OPEN_PAREN,
    CLOSE_PAREN,
    SPREAD,
    COLON,
    EQUALS,
    AT,
    OPEN_SQUARE,
    CLOSE_SQUARE,
    OPEN_CURLY,
    PIPE,
    CLOSE_CURLY,
    NAME,
    NEGATIVE_SIGN,
    NUMBER,
    EXPONENT_PART,
    FRACTIONAL_PART,
    QUOTE,
    BLOCK_QUOTE,
    WHITESPACE,
    ESCAPED_CHARACTER,
    ESCAPED_UNICODE,
    LINE_TERMINATOR,
    ON,
    OTHER,

    // Values
    NULL,
    TRUE,
    FALSE,

    // Keywords that aren't part of our lexing grammar
    QUERY_KEYWORD,
    MUTATION_KEYWORD,
    SUBSCRIPTION_KEYWORD,
    FRAGMENT_KEYWORD,

    // Compoosite nodes
    OPERATION_DEF,
    FRAGMENT_DEF,
    VARIABLE_DEFS,
    VARIABLE_DEF,
    VARIABLE,
    NAMED_TYPE,
    TYPE,
    SELECTION_SET,
    SELECTION,
    FIELD_SELECTION,
    ALIAS,
    ARGUMENTS,
    ARGUMENT,
    DEFAULT_VALUE,
    TYPE_CONDITION,
    FRAGMENT_SPREAD,
    FRAGMENT_NAME,
    INLINE_FRAGMENT,
    COMMENT,
    COMMENT_CONTENTS,
    DIRECTIVES,
    DIRECTIVE,
    ERROR,

    VALUE,
    INTEGER_VALUE,
    INTEGER_PART,
    BOOL_VALUE,
    FLOAT_VALUE,
    ENUM_VALUE,
    LIST_VALUE,
    OBJECT_VALUE,
    OBJECT_FIELD,
    STRING_VALUE,
    STRING_CONTENTS,

    // Root node.
    // Note that this needs to be last.
    ROOT,
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Lang {}

impl rowan::Language for Lang {
    type Kind = SyntaxKind;
    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        assert!(raw.0 <= SyntaxKind::ROOT as u16);
        unsafe { std::mem::transmute::<u16, SyntaxKind>(raw.0) }
    }
    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

pub type SyntaxNode = rowan::SyntaxNode<Lang>;
#[allow(unused)]
pub type SyntaxToken = rowan::SyntaxToken<Lang>;
#[allow(unused)]
pub type SyntaxElement = rowan::NodeOrToken<SyntaxNode, SyntaxToken>;

impl From<crate::lexer::Token> for SyntaxKind {
    fn from(token: crate::lexer::Token) -> Self {
        use crate::lexer::Token;
        use SyntaxKind::*;
        match token {
            Token::Comma => COMMA,
            Token::Hash => HASH,
            Token::Bang => BANG,
            Token::Dollar => DOLLAR,
            Token::OpenParen => OPEN_PAREN,
            Token::CloseParen => CLOSE_PAREN,
            Token::Spread => SPREAD,
            Token::Colon => COLON,
            Token::Equals => EQUALS,
            Token::At => AT,
            Token::OpenSquare => OPEN_SQUARE,
            Token::CloseSquare => CLOSE_SQUARE,
            Token::OpenCurly => OPEN_CURLY,
            Token::Pipe => PIPE,
            Token::CloseCurly => CLOSE_CURLY,
            Token::Name => NAME,
            Token::NegativeSign => NEGATIVE_SIGN,
            Token::ExponentPart => EXPONENT_PART,
            Token::FractionalPart => FRACTIONAL_PART,
            Token::Quote => QUOTE,
            Token::BlockQuote => BLOCK_QUOTE,
            Token::Whitespace => WHITESPACE,
            Token::EscapedUnicode => ESCAPED_UNICODE,
            Token::EscapedCharacter => ESCAPED_CHARACTER,
            Token::LineTerminator => LINE_TERMINATOR,

            // TODO: not sure about this one, but lets seee.
            Token::Other => OTHER,
            Token::Number => NUMBER,
        }
    }
}
