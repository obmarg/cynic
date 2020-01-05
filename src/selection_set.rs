use json_decode::Decoder;
use std::marker::PhantomData;

#[derive(Debug, PartialEq)]
enum Error {
    DecodeError(json_decode::DecodeError),
}

struct SelectionSet<'a, DecodesTo, TypeLock> {
    // Temporary type for fields - will probably needs something smarter at some point.
    fields: Vec<String>,

    decoder: Box<dyn Decoder<'a, DecodesTo> + 'a>,

    phantom: PhantomData<TypeLock>,
}

impl<'a, DecodesTo, TypeLock> SelectionSet<'a, DecodesTo, TypeLock> {
    fn decode(&self, value: &serde_json::Value) -> Result<DecodesTo, Error> {
        (*self.decoder).decode(value).map_err(Error::DecodeError)
    }
}

fn string<'a>() -> SelectionSet<'a, String, ()> {
    // TODO: probably need to do something about the inner fields?
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
    // TODO: probably need to do something about the inner fields?
    SelectionSet {
        fields: vec![field_name.to_string()],
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
        fields: param1.fields.clone(),
        decoder: json_decode::map1(func, param1.decoder),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct TestStruct {
        field_one: String,
    }

    impl TestStruct {
        fn new(field_one: String) -> Self {
            TestStruct {
                field_one: field_one,
            }
        }
    }

    struct RootQuery;

    struct Query;

    impl Query {
        fn field_one(s: SelectionSet<String, ()>) -> SelectionSet<String, RootQuery> {
            field("field_one", s)
        }
    }

    #[test]
    fn decode_using_dsl() {
        let selection_set: SelectionSet<_, RootQuery> =
            map1(TestStruct::new, Query::field_one(string()));

        let json = serde_json::from_str(r#"{"field_one": "test"}"#).unwrap();

        assert_eq!(
            selection_set.decode(&json),
            Ok(TestStruct {
                field_one: "test".to_string()
            })
        )
    }
}