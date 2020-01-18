use json_decode::Decoder;
use std::collections::HashMap;
use std::marker::PhantomData;

use crate::{field::Field, scalar, Argument, GraphQLResponse, QueryBody, QueryRoot};

#[derive(Debug, PartialEq)]
enum Error {
    DecodeError(json_decode::DecodeError),
}

pub struct SelectionSet<'a, DecodesTo, TypeLock> {
    fields: Vec<Field>,

    decoder: Box<dyn Decoder<'a, DecodesTo> + 'a>,

    phantom: PhantomData<TypeLock>,
}

// TODO: This definitely should live in a different file.
pub trait Query {
    type TypeLock;
    type ResponseData;

    fn body<'a>(&'a self) -> Result<QueryBody<'a>, ()>;
    fn decode_response(
        &self,
        response: GraphQLResponse<serde_json::Value>,
    ) -> Result<GraphQLResponse<Self::ResponseData>, json_decode::DecodeError>;
}

impl<'a, DecodesTo, TypeLock> Query for SelectionSet<'a, DecodesTo, TypeLock>
where
    TypeLock: QueryRoot,
{
    type TypeLock = TypeLock;
    type ResponseData = DecodesTo;

    fn body<'b>(&'b self) -> Result<QueryBody<'b>, ()> {
        (self as &SelectionSet<'a, DecodesTo, TypeLock>)
            .query_and_arguments()
            .map(|(query, arguments)| {
                let variables = arguments
                    .into_iter()
                    .enumerate()
                    .map(|(i, value)| (format!("${}", i), value))
                    .collect();

                QueryBody { query, variables }
            })
    }

    fn decode_response(
        &self,
        response: GraphQLResponse<serde_json::Value>,
    ) -> Result<GraphQLResponse<DecodesTo>, json_decode::DecodeError> {
        if let Some(data) = response.data {
            Ok(GraphQLResponse {
                // TODO: GET RID OF UNWRAP.  I am being extremely lazy by calling it.
                data: Some(self.decode(&data).unwrap()),
                errors: response.errors,
            })
        } else {
            Ok(GraphQLResponse {
                data: None,
                errors: response.errors,
            })
        }
    }
}

impl<'a, DecodesTo, TypeLock> SelectionSet<'a, DecodesTo, TypeLock> {
    fn decode(&self, value: &serde_json::Value) -> Result<DecodesTo, Error> {
        (*self.decoder).decode(value).map_err(Error::DecodeError)
    }

    fn query_and_arguments<'b>(&'b self) -> Result<(String, Vec<&'b serde_json::Value>), ()> {
        let mut arguments: Vec<Result<&serde_json::Value, ()>> = vec![];
        let query = self
            .fields
            .iter()
            .map(|f| f.query(0, 2, &mut arguments))
            .collect();

        let arguments: Vec<_> = arguments.into_iter().collect::<Result<Vec<_>, ()>>()?;

        Ok((query, arguments))
    }
}

pub fn string() -> SelectionSet<'static, String, ()> {
    SelectionSet {
        fields: vec![],
        decoder: json_decode::string(),
        phantom: PhantomData,
    }
}

pub fn integer() -> SelectionSet<'static, i64, ()> {
    SelectionSet {
        fields: vec![],
        decoder: json_decode::integer(),
        phantom: PhantomData,
    }
}

pub fn float() -> SelectionSet<'static, f64, ()> {
    SelectionSet {
        fields: vec![],
        decoder: json_decode::float(),
        phantom: PhantomData,
    }
}

pub fn boolean() -> SelectionSet<'static, bool, ()> {
    SelectionSet {
        fields: vec![],
        decoder: json_decode::boolean(),
        phantom: PhantomData,
    }
}

pub fn serde<T>() -> SelectionSet<'static, T, ()>
where
    for<'de> T: serde::Deserialize<'de>,
    T: 'static,
{
    SelectionSet {
        fields: vec![],
        decoder: json_decode::serde(),
        phantom: PhantomData,
    }
}

pub fn json() -> SelectionSet<'static, serde_json::Value, ()> {
    SelectionSet {
        fields: vec![],
        decoder: json_decode::json(),
        phantom: PhantomData,
    }
}

pub fn scalar<S>() -> SelectionSet<'static, S, ()>
where
    S: scalar::Scalar + 'static,
{
    SelectionSet {
        fields: vec![],
        decoder: scalar::decoder(),
        phantom: PhantomData,
    }
}

pub fn vec<'a, DecodesTo, TypeLock>(
    inner_selection: SelectionSet<'a, DecodesTo, TypeLock>,
) -> SelectionSet<'a, Vec<DecodesTo>, TypeLock>
where
    DecodesTo: 'a,
{
    SelectionSet {
        fields: inner_selection.fields,
        decoder: json_decode::list(inner_selection.decoder),
        phantom: PhantomData,
    }
}

pub fn option<'a, DecodesTo, TypeLock>(
    inner_selection: SelectionSet<'a, DecodesTo, TypeLock>,
) -> SelectionSet<'a, Option<DecodesTo>, TypeLock>
where
    DecodesTo: 'a,
{
    SelectionSet {
        fields: inner_selection.fields,
        decoder: json_decode::option(inner_selection.decoder),
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

pub fn field<'a, DecodesTo, TypeLock, InnerTypeLock>(
    field_name: &str,
    arguments: Vec<Argument>,
    selection_set: SelectionSet<'a, DecodesTo, InnerTypeLock>,
) -> SelectionSet<'a, DecodesTo, TypeLock>
where
    DecodesTo: 'a,
{
    let field = if selection_set.fields.is_empty() {
        Field::Leaf(field_name.to_string(), arguments)
    } else {
        Field::Composite(field_name.to_string(), arguments, selection_set.fields)
    };

    SelectionSet {
        fields: vec![field],
        decoder: json_decode::field(field_name, selection_set.decoder),
        phantom: PhantomData,
    }
}

pub use map as map1;

pub fn map<'a, F, T1, NewDecodesTo, TypeLock>(
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
        decoder: json_decode::map(func, param1.decoder),
    }
}

macro_rules! define_map {
    ($fn_name:ident, $($i:ident),+) => {
        pub fn $fn_name<'a, F, $($i, )+ NewDecodesTo, TypeLock>(
            func: F,
            $($i: SelectionSet<'a, $i, TypeLock>,)+
        ) -> SelectionSet<'a, NewDecodesTo, TypeLock>
        where
            F: Fn($($i, )+) -> NewDecodesTo + 'a,
            $($i: 'a,)+
            NewDecodesTo: 'a
        {
            let mut fields = Vec::new();
            $(
                fields.extend($i.fields.into_iter());
            )+

            SelectionSet {
                phantom: PhantomData,
                fields,
                decoder: json_decode::$fn_name(func, $($i.decoder, )+)
            }
        }
    };
}

define_map!(map2, _1, _2);
define_map!(map3, _1, _2, _3);
define_map!(map4, _1, _2, _3, _4);
define_map!(map5, _1, _2, _3, _4, _5);
define_map!(map6, _1, _2, _3, _4, _5, _6);
define_map!(map7, _1, _2, _3, _4, _5, _6, _7);
define_map!(map8, _1, _2, _3, _4, _5, _6, _7, _8);
define_map!(map9, _1, _2, _3, _4, _5, _6, _7, _8, _9);
define_map!(map10, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10);
define_map!(map11, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11);
define_map!(map12, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12);
define_map!(map13, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13);
define_map!(map14, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14);
define_map!(map15, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15);
define_map!(map16, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16);
define_map!(map17, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17);
define_map!(map18, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18);
define_map!(
    map19, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19
);
define_map!(
    map20, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19,
    _20
);

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
        use super::super::{field, string, Argument, SelectionSet};

        pub struct RootQuery;

        pub struct Query;

        pub struct QueryWithArgsArguments {
            pub required_arg: String,
        }

        #[derive(Default)]
        pub struct QueryWithArgsOptionals {
            pub opt_string: Option<String>,
        }

        impl Query {
            pub fn test_struct<'a, T>(
                fields: SelectionSet<'a, T, TestStruct>,
            ) -> SelectionSet<'a, T, RootQuery>
            where
                T: 'a,
            {
                field("test_struct", vec![], fields)
            }

            pub fn with_args<'a, T: 'a>(
                required: QueryWithArgsArguments,
                optionals: QueryWithArgsOptionals,
                fields: SelectionSet<'a, T, NestedStruct>,
            ) -> SelectionSet<'a, T, RootQuery> {
                let mut args = vec![Argument::new(
                    "required_arg",
                    serde_json::Value::String(required.required_arg),
                )];
                if optionals.opt_string.is_some() {
                    args.push(Argument::new(
                        "opt_string",
                        serde_json::Value::String(optionals.opt_string.unwrap()),
                    ));
                }
                field("nested", args, fields)
            }
        }

        pub struct TestStruct;

        impl TestStruct {
            pub fn field_one() -> SelectionSet<'static, String, TestStruct> {
                field("field_one", vec![], string())
            }

            pub fn nested<'a, T>(
                fields: SelectionSet<'a, T, NestedStruct>,
            ) -> SelectionSet<'a, T, TestStruct>
            where
                T: 'a,
            {
                field("nested", vec![], fields)
            }
        }

        pub struct NestedStruct;

        impl NestedStruct {
            pub fn a_string() -> SelectionSet<'static, String, NestedStruct> {
                field("a_string", vec![], string())
            }
        }
    }

    #[test]
    fn decode_using_dsl() {
        let selection_set: SelectionSet<_, query_dsl::RootQuery> = map(
            Query::new,
            query_dsl::Query::test_struct(map2(
                TestStruct::new,
                query_dsl::TestStruct::field_one(),
                query_dsl::TestStruct::nested(map(
                    NestedStruct::new,
                    query_dsl::NestedStruct::a_string(),
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
        let selection_set: SelectionSet<_, query_dsl::RootQuery> = map(
            Query::new,
            query_dsl::Query::test_struct(map2(
                TestStruct::new,
                query_dsl::TestStruct::field_one(),
                query_dsl::TestStruct::nested(map(
                    NestedStruct::new,
                    query_dsl::NestedStruct::a_string(),
                )),
            )),
        );

        assert_eq!(
            selection_set.query_and_arguments(),
            Ok((
                "test_struct {\n  field_one\n  nested {\n    a_string\n  }\n}\n".to_string(),
                vec![]
            ))
        )
    }

    #[test]
    fn test_vars_with_optionals_missing() {
        let selection_set: SelectionSet<Option<i32>, query_dsl::RootQuery> = map(
            |_| None,
            query_dsl::Query::with_args(
                query_dsl::QueryWithArgsArguments {
                    required_arg: "test".to_string(),
                },
                Default::default(),
                map(NestedStruct::new, query_dsl::NestedStruct::a_string()),
            ),
        );

        let (query, args) = selection_set.query_and_arguments().unwrap();
        assert_eq!(args.len(), 1);
    }

    fn test_vars_with_optionals_present() {
        let selection_set: SelectionSet<Option<i32>, query_dsl::RootQuery> = map(
            |_| None,
            query_dsl::Query::with_args(
                query_dsl::QueryWithArgsArguments {
                    required_arg: "test".to_string(),
                },
                query_dsl::QueryWithArgsOptionals {
                    opt_string: Some("test".to_string()),
                },
                map(NestedStruct::new, query_dsl::NestedStruct::a_string()),
            ),
        );

        let (query, args) = selection_set.query_and_arguments().unwrap();
        assert_eq!(args.len(), 1);
    }
}
