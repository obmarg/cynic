use json_decode::Decoder;
use std::marker::PhantomData;

#[derive(Debug, PartialEq)]
enum Error {
    DecodeError(json_decode::DecodeError),
}

enum Field {
    Leaf(String),
    Composite(String, Vec<Field>),
}

impl Field {
    fn query(&self, indent: usize, indent_size: usize) -> String {
        match self {
            Field::Leaf(field_name) => format!("{:indent$}{}\n", "", field_name, indent = indent),
            Field::Composite(field_name, child_fields) => {
                let child_query: String = child_fields
                    .iter()
                    .map(|f| f.query(indent + indent_size, indent_size))
                    .collect();

                format!(
                    "{0:indent$}{field_name} {{\n{child_query}{0:indent$}}}\n",
                    "",
                    field_name = field_name,
                    child_query = child_query,
                    indent = indent
                )
            }
        }
    }
}

pub struct SelectionSet<'a, DecodesTo, TypeLock> {
    fields: Vec<Field>,

    decoder: Box<dyn Decoder<'a, DecodesTo> + 'a>,

    phantom: PhantomData<TypeLock>,
}

impl<'a, DecodesTo, TypeLock> SelectionSet<'a, DecodesTo, TypeLock> {
    fn decode(&self, value: &serde_json::Value) -> Result<DecodesTo, Error> {
        (*self.decoder).decode(value).map_err(Error::DecodeError)
    }

    fn query(&self) -> String {
        self.fields.iter().map(|f| f.query(0, 2)).collect()
    }
}

fn string<'a>() -> SelectionSet<'a, String, ()> {
    SelectionSet {
        fields: vec![],
        decoder: json_decode::string(),
        phantom: PhantomData,
    }
}

// TODO: ok, so to fix this issue it seems like i can:
// 1. Make SelectionSet a trait, return specific types from each of these functions?
//      Except that won't fix this, as the issue is our SelectionSet _contains_ a dyn
//      which we can only behind a pointer, yet json_decode wants to own things.
// 2. Make json_decode take references?  Would probably need to do a lot of cloning
//      so not ideal.  Though if I changed from Box to Rc/Arc we'd be good for cloning...
// 3. Expose concrete types from json_decode and then do 1, including the decoder type
//      in the generic parameters...
// 4. Just make json_decode return Boxes/Rc's?

fn field<'a, DecodesTo, TypeLock, InnerTypeLock>(
    field_name: &str,
    selection_set: SelectionSet<'a, DecodesTo, InnerTypeLock>,
) -> SelectionSet<'a, DecodesTo, TypeLock>
where
    DecodesTo: 'a,
{
    let field = if selection_set.fields.is_empty() {
        Field::Leaf(field_name.to_string())
    } else {
        Field::Composite(field_name.to_string(), selection_set.fields)
    };

    SelectionSet {
        fields: vec![field],
        decoder: json_decode::field(field_name, selection_set.decoder),
        phantom: PhantomData,
    }
}

fn map1<'a, F, T1, NewDecodesTo, TypeLock>(
    func: F,
    param1: SelectionSet<'a, T1, TypeLock>,
) -> SelectionSet<'a, NewDecodesTo, TypeLock>
where
    F: Fn(T1) -> NewDecodesTo + 'a,
    T1: 'a,
    NewDecodesTo: 'a,
{
    SelectionSet {
        phantom: PhantomData,
        fields: param1.fields,
        decoder: json_decode::map1(func, param1.decoder),
    }
}

fn map2<'a, F, T1, T2, NewDecodesTo, TypeLock>(
    func: F,
    param1: SelectionSet<'a, T1, TypeLock>,
    param2: SelectionSet<'a, T2, TypeLock>,
) -> SelectionSet<'a, NewDecodesTo, TypeLock>
where
    F: Fn(T1, T2) -> NewDecodesTo + 'a,
    T1: 'a,
    T2: 'a,
    NewDecodesTo: 'a,
{
    let mut fields = Vec::with_capacity(param1.fields.len() + param2.fields.len());
    fields.extend(param1.fields.into_iter());
    fields.extend(param2.fields.into_iter());

    SelectionSet {
        phantom: PhantomData,
        fields,
        decoder: json_decode::map2(func, param1.decoder, param2.decoder),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Query {
        test_struct: TestStruct,
    }

    impl Query {
        fn new(test_struct: TestStruct) -> Self {
            Query { test_struct }
        }
    }

    #[derive(Debug, PartialEq)]
    struct TestStruct {
        field_one: String,
        nested: NestedStruct,
    }

    impl TestStruct {
        fn new(field_one: String, nested: NestedStruct) -> Self {
            TestStruct { field_one, nested }
        }
    }

    #[derive(Debug, PartialEq)]
    struct NestedStruct {
        a_string: String,
    }

    impl NestedStruct {
        fn new(a_string: String) -> Self {
            NestedStruct { a_string }
        }
    }

    mod query_dsl {
        use super::super::{field, SelectionSet};

        pub struct RootQuery;

        pub struct Query;

        impl Query {
            pub fn test_struct<'a, T>(
                fields: SelectionSet<'a, T, TestStruct>,
            ) -> SelectionSet<'a, T, RootQuery>
            where
                T: 'a,
            {
                field("test_struct", fields)
            }
        }

        pub struct TestStruct;

        impl TestStruct {
            pub fn field_one(inner: SelectionSet<String, ()>) -> SelectionSet<String, TestStruct> {
                field("field_one", inner)
            }

            pub fn nested<'a, T>(
                fields: SelectionSet<'a, T, NestedStruct>,
            ) -> SelectionSet<'a, T, TestStruct>
            where
                T: 'a,
            {
                field("nested", fields)
            }
        }

        pub struct NestedStruct;

        impl NestedStruct {
            pub fn a_string(inner: SelectionSet<String, ()>) -> SelectionSet<String, NestedStruct> {
                field("a_string", inner)
            }
        }
    }

    #[test]
    fn decode_using_dsl() {
        let selection_set: SelectionSet<_, query_dsl::RootQuery> = map1(
            Query::new,
            query_dsl::Query::test_struct(map2(
                TestStruct::new,
                query_dsl::TestStruct::field_one(string()),
                query_dsl::TestStruct::nested(map1(
                    NestedStruct::new,
                    query_dsl::NestedStruct::a_string(string()),
                )),
            )),
        );

        let json = serde_json::json!({"test_struct": {"field_one": "test", "nested": {"a_string": "hello"}}});

        assert_eq!(
            selection_set.decode(&json),
            Ok(Query {
                test_struct: TestStruct {
                    field_one: "test".to_string(),
                    nested: NestedStruct {
                        a_string: "hello".to_string()
                    }
                }
            })
        )
    }

    #[test]
    fn test_query_building() {
        let selection_set: SelectionSet<_, query_dsl::RootQuery> = map1(
            Query::new,
            query_dsl::Query::test_struct(map2(
                TestStruct::new,
                query_dsl::TestStruct::field_one(string()),
                query_dsl::TestStruct::nested(map1(
                    NestedStruct::new,
                    query_dsl::NestedStruct::a_string(string()),
                )),
            )),
        );

        assert_eq!(
            selection_set.query(),
            "test_struct {\n  field_one\n  nested {\n    a_string\n  }\n}\n"
        )
    }

}
