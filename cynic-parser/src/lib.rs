pub mod common;
pub mod executable;
pub mod type_system;
pub mod values;

mod errors;
mod lexer;
mod span;

#[allow(clippy::all)]
mod parser;

#[cfg(feature = "print")]
pub mod printing;

#[cfg(feature = "report")]
pub use errors::Report;

pub use self::{
    errors::Error,
    executable::ExecutableDocument,
    span::Span,
    type_system::TypeSystemDocument,
    values::{ConstValue, Value},
};

pub fn parse_type_system_document(input: &str) -> Result<TypeSystemDocument, Error> {
    let lexer = lexer::Lexer::new(input);
    let mut ast = type_system::writer::TypeSystemAstWriter::new();

    parser::TypeSystemDocumentParser::new().parse(input, &mut ast, lexer)?;

    Ok(ast.finish())
}

pub fn parse_executable_document(input: &str) -> Result<ExecutableDocument, Error> {
    let lexer = lexer::Lexer::new(input);
    let mut ast = executable::writer::ExecutableAstWriter::new();

    parser::ExecutableDocumentParser::new().parse(input, &mut ast, lexer)?;

    Ok(ast.finish())
}

trait AstLookup<Id> {
    type Output: ?Sized;

    fn lookup(&self, index: Id) -> &Self::Output;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        insta::assert_snapshot!(
            parse_type_system_document("schema { query:Query }").unwrap().to_sdl_pretty(),
            @r###"
        schema {
          query: Query
        }
        "###
        );
    }

    #[test]
    fn test_basic_object() {
        insta::assert_snapshot!(
            parse_type_system_document(r#"
                type MyType implements Blah & Bloo @hello {
                    field: Whatever @hello(name: ["string"]),
                    other: [[Int!]]!
                }"#
            ).unwrap().to_sdl_pretty(),
            @r###"
        type MyType implements Blah & Bloo @hello {
          field: Whatever @hello(name: ["string"])
          other: [[Int!]]!
        }
        "###
        );
    }

    #[test]
    fn test_basic_interface() {
        insta::assert_snapshot!(
            parse_type_system_document(r#"
                interface MyType implements Blah & Bloo @hello {
                    field: Whatever @hello(name: ["string"]),
                    other: [[Int!]]!
                }"#
            ).unwrap().to_sdl_pretty(),
            @r###"
        interface MyType implements Blah & Bloo @hello {
          field: Whatever @hello(name: ["string"])
          other: [[Int!]]!
        }
        "###
        );
    }

    #[test]
    fn test_basic_union() {
        insta::assert_snapshot!(
            parse_type_system_document(r#"
                union MyType = Blah | Bloo
                "#
            ).unwrap().to_sdl_pretty(),
            @r###"
        union MyType = Blah | Bloo

        "###
        );
    }

    #[test]
    fn test_basic_scalar() {
        insta::assert_snapshot!(
            parse_type_system_document(r#"
                scalar MyType @hello(there: [{thing: "other"}])
                "#
            ).unwrap().to_sdl_pretty(),
            @r###"
        scalar MyType @hello(there: [{ thing: "other" }])
        "###
        );
    }

    #[test]
    fn test_basic_enum() {
        insta::assert_snapshot!(
            parse_type_system_document(r#"
                enum MyEnum {
                    BLAH,
                    BLOO
                }
                "#
            ).unwrap().to_sdl_pretty(),
            @r###"
        enum MyEnum {
          BLAH
          BLOO
        }
        "###
        );
    }

    #[test]
    fn test_schema_field() {
        // Use a keyword as a field name and make sure it's fine
        insta::assert_snapshot!(
            parse_type_system_document( "type MyType { query: String }").unwrap().to_sdl_pretty(),
            @r###"
        type MyType {
          query: String
        }
        "###
        )
    }

    #[test]
    fn test_input() {
        insta::assert_snapshot!(
            parse_type_system_document(
                r#"
                "I am a description"
                input MyType @hello { query: String = "Hello" }
                "#
            ).unwrap().to_sdl_pretty(),
            @r###"
        "I am a description"
        input MyType @hello {
          query: String = "Hello"
        }
        "###
        );
    }
}
