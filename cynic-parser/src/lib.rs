use lalrpop_util::lalrpop_mod;

mod ast;
mod lexer;
mod printer;

// TODO: Rethink WrappingType, OperationType & DirectiveLocation at least?
pub use self::ast::{
    ids, readers, storage, writer::AstWriter, Ast, DirectiveLocation, OperationType, WrappingType,
};

lalrpop_mod!(schema);

pub fn parse_type_system_document(input: &str) -> Ast {
    let lexer = lexer::Lexer::new(input);
    let mut ast = AstWriter::new();

    schema::TypeSystemDocumentParser::new()
        .parse(input, &mut ast, lexer)
        .expect("TODO: error handling");

    ast.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        insta::assert_snapshot!(
            parse_type_system_document("schema { query:Query }").to_sdl(),
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
            ).to_sdl(),
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
            ).to_sdl(),
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
            ).to_sdl(),
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
            ).to_sdl(),
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
            ).to_sdl(),
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
            parse_type_system_document( "type MyType { query: String }").to_sdl(),
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
            ).to_sdl(),
            @r###"
        "I am a description"
        input MyType @hello {
          query: String = "Hello"
        }
        "###
        );
    }
}
