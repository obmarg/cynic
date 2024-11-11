#![allow(clippy::disallowed_names)]

use cynic_parser::type_system::Definition;
use cynic_parser_deser::{ConstDeserializer, DeserValue, ValueDeserialize, ValueDeserializeOwned};

#[derive(ValueDeserialize)]
#[deser(default)]
struct StructDefault {
    foo: Option<usize>,
}

#[test]
fn test_struct_default() {
    assert_eq!(deser::<StructDefault>("@id").unwrap().foo, None);
    assert_eq!(
        deser::<StructDefault>("@id(foo: 10)").unwrap().foo,
        Some(10)
    );
}

#[derive(ValueDeserialize)]
struct FieldDefault {
    #[deser(default)]
    foo: Option<usize>,
}

#[test]
fn test_field_default() {
    assert_eq!(deser::<FieldDefault>("@id").unwrap().foo, None);
    assert_eq!(deser::<FieldDefault>("@id(foo: 10)").unwrap().foo, Some(10));
}

#[derive(ValueDeserialize)]
struct LiteralDefault {
    #[deser(default = 10)]
    foo: usize,
}

#[test]
fn test_literal_default() {
    assert_eq!(deser::<LiteralDefault>("@id").unwrap().foo, 10);
    assert_eq!(deser::<LiteralDefault>("@id(foo: 15)").unwrap().foo, 15);
}

#[derive(ValueDeserialize)]
struct WithTest {
    #[deser(with = deser_func_for_with)]
    foo: usize,
}

fn deser_func_for_with(_: DeserValue<'_>) -> Result<usize, cynic_parser_deser::Error> {
    Ok(100)
}

#[test]
fn test_with() {
    assert_eq!(deser::<WithTest>("@id(foo: 25)").unwrap().foo, 100);
}

fn deser<T>(input: &str) -> Result<T, cynic_parser_deser::Error>
where
    T: ValueDeserializeOwned,
{
    let doc =
        cynic_parser::parse_type_system_document(&format!("type Object {input} {{name: String}}"))
            .unwrap();
    let Definition::Type(definition) = doc.definitions().next().unwrap() else {
        unreachable!()
    };
    definition.directives().next().unwrap().deserialize::<T>()
}
