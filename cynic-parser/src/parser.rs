use std::any::Any;

use logos::{Logos, Span};
use rowan::{GreenNode, GreenNodeBuilder};

use crate::{
    lexer::Token,
    syntax::{
        SyntaxKind::{self, *},
        SyntaxNode,
    },
};

/// The parse results are stored as a "green tree".
/// We'll discuss working with the results later
pub struct Parse {
    green_node: GreenNode,
    #[allow(unused)]
    errors: Vec<(String, logos::Span)>,
}

pub fn parse(text: &str) -> Parse {
    let mut lexer = Token::lexer(text);
    let mut tokens = Vec::new();
    while let Some(token) = lexer.next() {
        tokens.push((token, lexer.slice(), lexer.span()));
    }
    tokens.reverse();

    Parser {
        tokens,
        builder: GreenNodeBuilder::new(),
        errors: Vec::new(),
    }
    .parse()
}

impl Parse {
    fn syntax(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.green_node.clone())
    }
}

struct Parser<'source> {
    tokens: Vec<(Token, &'source str, Span)>,
    builder: GreenNodeBuilder<'static>,

    // TODO: Do I want spanned errors?
    errors: Vec<(String, Span)>,
}

impl<'source> Parser<'source> {
    fn parse(mut self) -> Parse {
        self.builder.start_node(ROOT.into());
        loop {
            // Ok, so parsing out definitions here.
            let mut current_error = false;
            match executable_def(&mut self) {
                Res::Eof => {
                    break;
                }
                Res::ExpectedExecutableDef => {
                    self.builder.start_node(ERROR.into());
                    if !current_error {
                        self.error("expected executable definition");
                        current_error = true;
                    }
                    self.bump();
                    // TODO: better nodes somehow
                    self.builder.finish_node();
                }
                Res::Ok => {
                    current_error = false;
                }
            }
        }
        self.skip_ws();
        self.builder.finish_node();

        Parse {
            green_node: self.builder.finish(),
            errors: self.errors,
        }
    }

    fn current(&self) -> Option<Token> {
        self.tokens.last().map(|(t, _, _)| *t)
    }

    fn peek_next_non_ws(&self) -> Option<Token> {
        let mut rev_iter = self.tokens.iter().rev();
        rev_iter.next(); // Skip the first one.
        rev_iter
            .skip_while(|(t, _, _)| *t == Token::Whitespace || *t == Token::LineTerminator)
            .next()
            .map(|(t, _, _)| *t)
    }

    fn current_str(&self) -> Option<&'source str> {
        self.tokens.last().map(|(_, s, _)| *s)
    }

    fn current_pair(&self) -> Option<(Token, &'source str)> {
        self.tokens.last().map(|(t, s, _)| (*t, *s))
    }

    fn current_span(&self) -> Option<Span> {
        self.tokens.last().map(|(_, _, span)| span.clone())
    }

    fn error(&mut self, err: impl Into<String>) {
        self.errors
            .push((err.into(), self.current_span().expect("TODO: handle this")));
    }

    fn bump_as(&mut self, kind: SyntaxKind) {
        let (_, text, _) = self.tokens.pop().unwrap();
        self.builder.token(kind.into(), text)
    }

    fn bump(&mut self) {
        let (kind, text, _) = self.tokens.pop().unwrap();
        self.builder.token(SyntaxKind::from(kind).into(), text);
    }

    fn skip_ws(&mut self) {
        while self.current() == Some(Token::Whitespace)
            || self.current() == Some(Token::LineTerminator)
        {
            self.bump();
        }
    }
}

enum Res {
    Eof,
    Ok,
    ExpectedExecutableDef,
}

fn executable_def(parser: &mut Parser) -> Res {
    parser.skip_ws();
    match parser.current_pair() {
        None => {
            // Return some EOF indicator
            return Res::Eof;
        }
        Some((Token::Name, "query")) => {
            parser.builder.start_node(OPERATION_DEF.into());
            parser.bump_as(QUERY_KEYWORD);
            operation(parser);
            parser.builder.finish_node();
            return Res::Ok;
        }
        Some((Token::Name, "mutation")) => {
            parser.builder.start_node(OPERATION_DEF.into());
            parser.bump_as(MUTATION_KEYWORD);
            operation(parser);
            parser.builder.finish_node();
            return Res::Ok;
        }
        Some((Token::Name, "subscription")) => {
            parser.builder.start_node(OPERATION_DEF.into());
            parser.bump_as(SUBSCRIPTION_KEYWORD);
            operation(parser);
            parser.builder.finish_node();
            return Res::Ok;
        }
        Some((Token::OpenCurly, _)) => {
            parser.builder.start_node(OPERATION_DEF.into());
            selection_set(parser);
            parser.builder.finish_node();
            return Res::Ok;
        }
        Some((Token::Name, "fragment")) => {
            parser.builder.start_node(FRAGMENT_DEF.into());
            parser.bump_as(FRAGMENT_KEYWORD);
            fragment(parser);
            parser.builder.finish_node();
            return Res::Ok;
        }
        other => {
            return Res::ExpectedExecutableDef;
        }
    }
}

fn operation(parser: &mut Parser) {
    parser.skip_ws();
    if let Some(Token::Name) = parser.current() {
        parser.bump();
    }

    parser.skip_ws();
    if let Some(Token::OpenParen) = parser.current() {
        variable_defs(parser);
    }

    // TODO: directives

    parser.skip_ws();
    if let Some(Token::OpenCurly) = parser.current() {
        selection_set(parser);
    } else {
        todo!()
        // TODO: Error
    }
}

fn variable_defs(parser: &mut Parser) {
    assert_eq!(parser.current(), Some(Token::OpenParen));
    parser.builder.start_node(VARIABLE_DEFS.into());
    parser.bump();
    loop {
        parser.skip_ws();
        match parser.current() {
            None => break,
            Some(Token::Dollar) => {
                parser.builder.start_node(VARIABLE_DEF.into());
                variable(parser);
                parser.skip_ws();
                match parser.current() {
                    Some(Token::Colon) => {
                        parser.bump();
                    }
                    _ => parser.error("expected :"),
                }
                type_(parser);
                parser.builder.finish_node();
                // todo: default values
            }
            Some(Token::CloseParen) => {
                parser.bump();
                break;
            }
            _ => {
                todo!()
                // TODO: error
            }
        }
    }
    parser.builder.finish_node();
}

fn variable(parser: &mut Parser) {
    assert_eq!(parser.current(), Some(Token::Dollar));

    parser.builder.start_node(VARIABLE.into());
    parser.bump();
    match parser.current() {
        Some(Token::Name) => parser.bump(),
        _ => parser.error("expected name"),
    }
    parser.builder.finish_node();
}

fn type_(parser: &mut Parser) {
    parser.skip_ws();
    match parser.current() {
        Some(Token::Name) => {
            parser.builder.start_node(TYPE.into());
            named_type(parser);
            if let Some(Token::Bang) = parser.current() {
                parser.bump();
            }
            parser.builder.finish_node();
        }
        Some(Token::OpenSquare) => {
            parser.builder.start_node(TYPE.into());
            parser.bump();
            type_(parser);
            match parser.current() {
                Some(Token::CloseSquare) => {
                    parser.bump();
                }
                _ => {
                    parser.error("expected ]");
                }
            }
            parser.skip_ws();
            if let Some(Token::Bang) = parser.current() {
                parser.bump()
            }
            parser.builder.finish_node();
        }
        _ => {
            parser.error("expected name or [");
        }
    }
    parser.skip_ws();
}

fn named_type(parser: &mut Parser) {
    assert_eq!(parser.current(), Some(Token::Name));
    parser.builder.start_node(NAMED_TYPE.into());
    parser.bump();
    parser.builder.finish_node();
}

fn selection_set(parser: &mut Parser) {
    assert_eq!(parser.current(), Some(Token::OpenCurly));

    parser.builder.start_node(SELECTION_SET.into());
    parser.bump();

    loop {
        parser.skip_ws();
        match parser.current() {
            None => {
                parser.error("expected selection");
                break;
            }
            Some(Token::Name) => {
                parser.builder.start_node(SELECTION.into());
                field_selection(parser);
                parser.builder.finish_node();
            }
            Some(Token::CloseCurly) => {
                parser.bump();
                break;
            }
            Some(Token::Spread) => {
                // Fragment or inline fragment
                todo!()
            }
            _ => {
                // TODO: is this good? not sure it is..
                parser.error("expected selection");
                break;
            }
        }
    }

    parser.builder.finish_node();
}

fn field_selection(parser: &mut Parser) {
    assert_eq!(parser.current(), Some(Token::Name));
    parser.builder.start_node(FIELD_SELECTION.into());
    if let Some(Token::Colon) = parser.peek_next_non_ws() {
        parser.builder.start_node(ALIAS.into());

        // Take the name & colon
        parser.bump();
        parser.skip_ws();
        parser.bump();
        parser.builder.finish_node();
    }

    parser.skip_ws();
    match parser.current() {
        Some(Token::Name) => {
            parser.bump();
        }
        _ => parser.error("expected name"),
    }

    parser.skip_ws();
    if let Some(Token::OpenParen) = parser.current() {
        arguments(parser);
    }

    parser.skip_ws();
    if let Some(Token::At) = parser.current() {
        todo!()
        // TODO: parse directives
    }

    parser.skip_ws();
    if let Some(Token::OpenCurly) = parser.current() {
        selection_set(parser)
    }

    parser.builder.finish_node();
}

fn arguments(parser: &mut Parser) {
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
                break;
            }
            _ => {
                // TODO: Is this good?  no idea
                parser.error("expected a name or )");
                break;
            }
        }
    }
}

fn argument(parser: &mut Parser) {
    assert_eq!(parser.current(), Some(Token::Name));
    todo!()
    // TODO: don't forget to eat a comma if there is one
}

fn fragment(parser: &mut Parser) {
    parser.skip_ws();
    match parser.current() {
        None => parser.error("expected fragment name"),
        Some(Token::Name) if parser.current_str() != Some("on") => {
            parser.bump();
        }
        _ => {
            parser.error("expected fragment name");
        }
    }

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
        other => {
            parser.error("expected a type condition");
        }
    }

    // TODO: Directives

    parser.skip_ws();
    if let Some(Token::OpenCurly) = parser.current() {
        selection_set(parser);
    } else {
        todo!()
        // TODO: Error
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read};

    use rstest::rstest;

    use super::*;

    #[test]
    fn test_simple_parse() {
        let query = r#"
query {
    posts {
        title
        content
    }
}
"#;
        let result = parse(query);
        assert_eq!(result.errors, vec![]);
        assert_eq!(result.green_node.to_string(), query);
        insta::assert_debug_snapshot!(result.syntax());
    }

    #[rstest]
    #[case::minimal("tests/queries/minimal.graphql")]
    #[case::minimal_mutation("tests/queries/minimal_mutation.graphql")]
    #[case::minimal_query("tests/queries/minimal_query.graphql")]
    #[case::named_query("tests/queries/named_query.graphql")]
    #[case::nested_selection("tests/queries/nested_selection.graphql")]
    #[case::query_aliases("tests/queries/query_aliases.graphql")]
    #[case::query_vars("tests/queries/query_vars.graphql")]
    #[case::fragment("tests/queries/fragment.graphql")]
    fn test_query_file(#[case] file: String) {
        let mut query = String::new();
        File::open(file)
            .unwrap()
            .read_to_string(&mut query)
            .unwrap();

        let result = parse(&query);

        assert_eq!(result.errors, vec![]);
        assert_eq!(result.green_node.to_string(), query);
        insta::assert_debug_snapshot!(result.syntax());
    }
}
