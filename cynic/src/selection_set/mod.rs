//! `SelectionSet`s are the core building block of GraphQL queries in cynic.
//!
//! A `SelectionSet` represents part of a query with one or more fields that
//! should be queried, and details of how those fields should be deserialized
//! after the query is run.
//!
//! The functions in this module fall into a few categories:
//!
//! 1. Functions that decode scalar values, such as `int` & `bool`.
//! 2. Functions that decode container types such as `vec` & `option`.  These
//!    take a selection set that is used to decode the contents of the container.
//! 3. The `field` function, which selects a particular field of an object, and
//!    uses another selection set to decode the type of that object.
//! 4. Combinators like the `map`, `map2` etc, functions that combine multiple
//!    selection sets into a single type.
//!
//! Cynic provides Query DSL generation & derive macros that mean for many cases you
//! shouldn't need to use the functions in this module directly.  However for more
//! advanced use cases (or if you dislike macros) these can still be useful.

mod field;

use json_decode::{BoxDecoder, DecodeError};
use std::collections::HashMap;
use std::marker::PhantomData;

use crate::{scalar, Argument, MutationRoot, QueryRoot};

use field::{Field, OperationType};

/// A marker trait used to encode GraphQL subtype relationships into the Rust
/// typesystem.
pub trait HasSubtype<Subtype> {}

/// A `SelectionSet` is a combination of a set of fields to fetch as part of a
/// GraphQL query and a decoder function that will decode the results of that
/// query.
///
/// Each `SelectionSet` has two generic parmeters:
///
/// - `DecodesTo` is the type that the selection set will decode when it is fed
///   the results of it's query.
/// - `TypeLock` is used to enforce type safety.  It allows the `query_dsl`
///   functionality in cynic to annotate each SelectionSet it returns such
///   that you can't build incorrect queries.
pub struct SelectionSet<'a, DecodesTo, TypeLock> {
    fields: Vec<Field>,

    pub(crate) decoder: BoxDecoder<'a, DecodesTo>,

    phantom: PhantomData<TypeLock>,
}

impl<'a, DecodesTo, TypeLock> SelectionSet<'a, DecodesTo, TypeLock> {
    /// Maps a `SelectionSet<_, DecodesTo, _>` to a `SelectionSet<_, R, _>` by applying
    /// the provided function `f` to the decoded data when decoding query results.
    ///
    /// For example, to fetch a string and then lowercase it:
    ///
    /// ```rust
    /// # use cynic::selection_set::{field, string};
    /// string().map(|s| s.to_lowercase());
    /// ```
    pub fn map<F, R>(self, f: F) -> SelectionSet<'a, R, TypeLock>
    where
        F: (Fn(DecodesTo) -> R) + 'a + Sync + Send,
        DecodesTo: 'a,
        R: 'a,
    {
        SelectionSet {
            fields: self.fields,
            decoder: json_decode::map(f, self.decoder),
            phantom: PhantomData,
        }
    }

    /// Creates a `SelectionSet` that depends on previous resutls.
    ///
    /// For example, to decode a different type depending on the value of
    /// another field:
    ///
    /// ```rust
    /// # use cynic::selection_set::{field, string, fail};
    /// field::<_, (), ()>("__typename", vec![], string())
    ///     .and_then(|typename| match typename.as_ref() {
    ///         "Cat" => field("cat", vec![], string()),
    ///         "Dog" => field("dog", vec![], string()),
    ///         _ => fail("")
    ///     });
    /// ```
    pub fn and_then<F, R>(self, f: F) -> SelectionSet<'a, R, TypeLock>
    where
        F: (Fn(DecodesTo) -> SelectionSet<'a, R, TypeLock>) + 'a + Sync + Send,
        DecodesTo: 'a,
        R: 'a,
    {
        let boxed_func = Box::new(f);
        SelectionSet {
            fields: self.fields,
            decoder: json_decode::and_then(move |value| (*boxed_func)(value).decoder, self.decoder),
            phantom: PhantomData,
        }
    }

    /// Changes the `TypeLock` on a `SelectionSet`.
    ///
    /// This is used when querying for an interface or a union type where you have
    /// a `SelectionSet` type locked to a subtype of an interface and want to use
    /// get a `SelectionSet` compatible with a field that has the type of the
    /// interface.
    pub fn transform_typelock<NewLock>(self) -> SelectionSet<'a, DecodesTo, NewLock>
    where
        NewLock: HasSubtype<TypeLock>,
    {
        SelectionSet {
            fields: self.fields,
            decoder: self.decoder,
            phantom: PhantomData,
        }
    }

    #[cfg(test)]
    fn decode(&self, value: &serde_json::Value) -> Result<DecodesTo, DecodeError> {
        (*self.decoder).decode(value)
    }

    pub(crate) fn query_arguments_and_decoder(
        self,
    ) -> (String, Vec<Argument>, BoxDecoder<'a, DecodesTo>) {
        let mut arguments: Vec<Argument> = vec![];
        let query = self
            .fields
            .into_iter()
            .map(|f| f.query(0, 2, &mut arguments))
            .collect();

        (query, arguments, self.decoder)
    }
}

/// Creates a `SelectionSet` that will decode a `String`
pub fn string() -> SelectionSet<'static, String, ()> {
    SelectionSet {
        fields: vec![],
        decoder: json_decode::string(),
        phantom: PhantomData,
    }
}

/// Creates a `SelectionSet` that will decode an `i32`
pub fn integer() -> SelectionSet<'static, i32, ()> {
    SelectionSet {
        fields: vec![],
        decoder: json_decode::integer(),
        phantom: PhantomData,
    }
}

/// Creates a `SelectionSet` that will decode an `f64`
pub fn float() -> SelectionSet<'static, f64, ()> {
    SelectionSet {
        fields: vec![],
        decoder: json_decode::float(),
        phantom: PhantomData,
    }
}

/// Creates a `SelectionSet` that will decode a `bool`
pub fn boolean() -> SelectionSet<'static, bool, ()> {
    SelectionSet {
        fields: vec![],
        decoder: json_decode::boolean(),
        phantom: PhantomData,
    }
}

/// Creates a `SelectionSet` that will decode a type that implements `serde::Deserialize`
pub fn serde<T>() -> SelectionSet<'static, T, ()>
where
    for<'de> T: serde::Deserialize<'de>,
    T: 'static + Send + Sync,
{
    SelectionSet {
        fields: vec![],
        decoder: json_decode::serde(),
        phantom: PhantomData,
    }
}

/// Creates a `SelectionSet` that will decode into a `serde_json::Value`
pub fn json() -> SelectionSet<'static, serde_json::Value, ()> {
    SelectionSet {
        fields: vec![],
        decoder: json_decode::json(),
        phantom: PhantomData,
    }
}

/// Creates a `SelectionSet` that will decode a type that implements `Scalar`
pub fn scalar<S>() -> SelectionSet<'static, S, ()>
where
    S: scalar::Scalar + 'static + Send + Sync,
{
    SelectionSet {
        fields: vec![],
        decoder: scalar::decoder(),
        phantom: PhantomData,
    }
}

/// Creates a `SelectionSet` that decodes a Vec of `inner_selection`
pub fn vec<'a, DecodesTo, TypeLock>(
    inner_selection: SelectionSet<'a, DecodesTo, TypeLock>,
) -> SelectionSet<'a, Vec<DecodesTo>, TypeLock>
where
    DecodesTo: 'a + Send + Sync,
{
    SelectionSet {
        fields: inner_selection.fields,
        decoder: json_decode::list(inner_selection.decoder),
        phantom: PhantomData,
    }
}

/// Creates a `SelectionSet` that decodes a nullable into an Option
pub fn option<'a, DecodesTo, TypeLock>(
    inner_selection: SelectionSet<'a, DecodesTo, TypeLock>,
) -> SelectionSet<'a, Option<DecodesTo>, TypeLock>
where
    DecodesTo: 'a + Send + Sync,
{
    SelectionSet {
        fields: inner_selection.fields,
        decoder: json_decode::option(inner_selection.decoder),
        phantom: PhantomData,
    }
}

/// Selects a field from a GraphQL object, decoding it with another `SelectionSet`
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

/// Creates a SelectionSet that adds some inline fragments to a query.
///
/// This should be provided a Vec of typenames to the selection set that should
/// be applied if that type is found.
pub fn inline_fragments<'a, DecodesTo, TypeLock>(
    fragments: Vec<(String, SelectionSet<'a, DecodesTo, TypeLock>)>,
) -> SelectionSet<'a, DecodesTo, TypeLock>
where
    DecodesTo: 'a + Send + Sync,
{
    let mut fields = vec![];
    let mut decoders = HashMap::new();

    fields.push(Field::Leaf("__typename".to_string(), vec![]));

    for (fragment_type, selection_set) in fragments {
        fields.push(Field::InlineFragment(
            fragment_type.to_string(),
            selection_set.fields,
        ));
        decoders.insert(fragment_type, selection_set.decoder);
    }

    SelectionSet {
        fields: fields,
        decoder: Box::new(FragmentDecoder {
            decoders,
            backup_decoder: None,
        }),
        phantom: PhantomData,
    }
}

struct FragmentDecoder<'a, DecodesTo> {
    decoders: HashMap<String, BoxDecoder<'a, DecodesTo>>,
    backup_decoder: Option<BoxDecoder<'a, DecodesTo>>,
}

impl<'a, DecodesTo> json_decode::Decoder<'a, DecodesTo> for FragmentDecoder<'a, DecodesTo> {
    fn decode(&self, value: &serde_json::Value) -> Result<DecodesTo, DecodeError> {
        let typename = value["__typename"].as_str().ok_or_else(|| {
            json_decode::DecodeError::MissingField("__typename".into(), value.to_string())
        })?;

        if let Some(decoder) = self.decoders.get(typename) {
            decoder.decode(value)
        } else if let Some(backup_decoder) = &self.backup_decoder {
            backup_decoder.decode(value)
        } else {
            Err(json_decode::DecodeError::Other(format!(
                "Unknown __typename: {}",
                typename,
            )))
        }
    }
}

pub(crate) fn query_root<'a, DecodesTo, InnerTypeLock: QueryRoot>(
    selection_set: SelectionSet<'a, DecodesTo, InnerTypeLock>,
) -> SelectionSet<'a, DecodesTo, ()>
where
    DecodesTo: 'a,
{
    SelectionSet {
        fields: vec![Field::Root(selection_set.fields, OperationType::Query)],
        decoder: selection_set.decoder,
        phantom: PhantomData,
    }
}

pub(crate) fn mutation_root<'a, DecodesTo, InnerTypeLock: MutationRoot>(
    selection_set: SelectionSet<'a, DecodesTo, InnerTypeLock>,
) -> SelectionSet<'a, DecodesTo, ()>
where
    DecodesTo: 'a,
{
    SelectionSet {
        fields: vec![Field::Root(selection_set.fields, OperationType::Mutation)],
        decoder: selection_set.decoder,
        phantom: PhantomData,
    }
}

/// Applies a function to the result of a selection.
pub use map as map1;

/// Applies a function to the result of a selection.
pub fn map<'a, F, T1, NewDecodesTo, TypeLock>(
    func: F,
    param1: SelectionSet<'a, T1, TypeLock>,
) -> SelectionSet<'a, NewDecodesTo, TypeLock>
where
    F: Fn(T1) -> NewDecodesTo + 'a + Send + Sync,
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
        /// Applies a function to the result of some SelectionSets.
        ///
        /// This can be used to create structs from the SelectionSets of their fields.
        /// For example, to create a user with three fields we would use the `map3` function:
        ///
        /// ```
        /// # use cynic::selection_set::{field, map3, string, integer};
        /// struct User {
        ///     id: i32,
        ///     name: String,
        ///     email: String,
        /// }
        ///
        /// impl User {
        ///     fn new(id: i32, name: String, email: String) -> User {
        ///         User { id, name, email }
        ///     }
        /// }
        ///
        /// map3(
        ///     User::new,
        ///     field::<_, (), ()>("id", vec![], integer()),
        ///     field::<_, (), ()>("email", vec![], string()),
        ///     field::<_, (), ()>("email", vec![], string()),
        /// );
        /// ```
        pub fn $fn_name<'a, F, $($i, )+ NewDecodesTo, TypeLock>(
            func: F,
            $($i: SelectionSet<'a, $i, TypeLock>,)+
        ) -> SelectionSet<'a, NewDecodesTo, TypeLock>
        where
            F: Fn($($i, )+) -> NewDecodesTo + 'a + Send + Sync,
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
define_map!(
    map21, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19,
    _20, _21
);
define_map!(
    map22, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19,
    _20, _21, _22
);
define_map!(
    map23, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19,
    _20, _21, _22, _23
);
define_map!(
    map24, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19,
    _20, _21, _22, _23, _24
);
define_map!(
    map25, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19,
    _20, _21, _22, _23, _24, _25
);
define_map!(
    map26, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19,
    _20, _21, _22, _23, _24, _25, _26
);
define_map!(
    map27, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19,
    _20, _21, _22, _23, _24, _25, _26, _27
);
define_map!(
    map28, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19,
    _20, _21, _22, _23, _24, _25, _26, _27, _28
);
define_map!(
    map29, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19,
    _20, _21, _22, _23, _24, _25, _26, _27, _28, _29
);
define_map!(
    map30, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19,
    _20, _21, _22, _23, _24, _25, _26, _27, _28, _29, _30
);
define_map!(
    map31, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19,
    _20, _21, _22, _23, _24, _25, _26, _27, _28, _29, _30, _31
);
define_map!(
    map32, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19,
    _20, _21, _22, _23, _24, _25, _26, _27, _28, _29, _30, _31, _32
);
define_map!(
    map33, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19,
    _20, _21, _22, _23, _24, _25, _26, _27, _28, _29, _30, _31, _32, _33
);
define_map!(
    map34, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19,
    _20, _21, _22, _23, _24, _25, _26, _27, _28, _29, _30, _31, _32, _33, _34
);
define_map!(
    map35, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19,
    _20, _21, _22, _23, _24, _25, _26, _27, _28, _29, _30, _31, _32, _33, _34, _35
);
define_map!(
    map36, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19,
    _20, _21, _22, _23, _24, _25, _26, _27, _28, _29, _30, _31, _32, _33, _34, _35, _36
);
define_map!(
    map37, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19,
    _20, _21, _22, _23, _24, _25, _26, _27, _28, _29, _30, _31, _32, _33, _34, _35, _36, _37
);
define_map!(
    map38, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19,
    _20, _21, _22, _23, _24, _25, _26, _27, _28, _29, _30, _31, _32, _33, _34, _35, _36, _37, _38
);
define_map!(
    map39, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19,
    _20, _21, _22, _23, _24, _25, _26, _27, _28, _29, _30, _31, _32, _33, _34, _35, _36, _37, _38,
    _39
);
define_map!(
    map40, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19,
    _20, _21, _22, _23, _24, _25, _26, _27, _28, _29, _30, _31, _32, _33, _34, _35, _36, _37, _38,
    _39, _40
);
define_map!(
    map41, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19,
    _20, _21, _22, _23, _24, _25, _26, _27, _28, _29, _30, _31, _32, _33, _34, _35, _36, _37, _38,
    _39, _40, _41
);
define_map!(
    map42, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19,
    _20, _21, _22, _23, _24, _25, _26, _27, _28, _29, _30, _31, _32, _33, _34, _35, _36, _37, _38,
    _39, _40, _41, _42
);
define_map!(
    map43, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19,
    _20, _21, _22, _23, _24, _25, _26, _27, _28, _29, _30, _31, _32, _33, _34, _35, _36, _37, _38,
    _39, _40, _41, _42, _43
);
define_map!(
    map44, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19,
    _20, _21, _22, _23, _24, _25, _26, _27, _28, _29, _30, _31, _32, _33, _34, _35, _36, _37, _38,
    _39, _40, _41, _42, _43, _44
);
define_map!(
    map45, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19,
    _20, _21, _22, _23, _24, _25, _26, _27, _28, _29, _30, _31, _32, _33, _34, _35, _36, _37, _38,
    _39, _40, _41, _42, _43, _44, _45
);
define_map!(
    map46, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19,
    _20, _21, _22, _23, _24, _25, _26, _27, _28, _29, _30, _31, _32, _33, _34, _35, _36, _37, _38,
    _39, _40, _41, _42, _43, _44, _45, _46
);
define_map!(
    map47, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19,
    _20, _21, _22, _23, _24, _25, _26, _27, _28, _29, _30, _31, _32, _33, _34, _35, _36, _37, _38,
    _39, _40, _41, _42, _43, _44, _45, _46, _47
);
define_map!(
    map48, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19,
    _20, _21, _22, _23, _24, _25, _26, _27, _28, _29, _30, _31, _32, _33, _34, _35, _36, _37, _38,
    _39, _40, _41, _42, _43, _44, _45, _46, _47, _48
);
define_map!(
    map49, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19,
    _20, _21, _22, _23, _24, _25, _26, _27, _28, _29, _30, _31, _32, _33, _34, _35, _36, _37, _38,
    _39, _40, _41, _42, _43, _44, _45, _46, _47, _48, _49
);
define_map!(
    map50, _1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, _18, _19,
    _20, _21, _22, _23, _24, _25, _26, _27, _28, _29, _30, _31, _32, _33, _34, _35, _36, _37, _38,
    _39, _40, _41, _42, _43, _44, _45, _46, _47, _48, _49, _50
);

/// Creates a `SelectionSet` that always decodes succesfully to a particular value
///
/// This is handy when used with `SelectionSet::and_then` - you can return a specific
/// hard coded value in some case.
pub fn succeed<'a, V, TypeLock>(value: V) -> SelectionSet<'a, V, TypeLock>
where
    V: Clone + Send + Sync + 'a,
{
    SelectionSet {
        fields: vec![],
        decoder: json_decode::succeed(value),
        phantom: PhantomData,
    }
}

/// Creates a `SelectionSet` that always decodes succesfully to the result of a function
///
/// This is similar to `succeeed` but can be used in cases where the type you wish to
/// return does not impl Clone.
pub fn succeed_using<'a, F, V, TypeLock>(f: F) -> SelectionSet<'a, V, TypeLock>
where
    F: Fn() -> V + Send + Sync + 'a,
    V: Send + Sync + 'a,
{
    SelectionSet {
        fields: vec![],
        decoder: json_decode::map(
            move |_| f(),
            json_decode::succeed::<Option<core::convert::Infallible>>(None),
        ),
        phantom: PhantomData,
    }
}

/// Creates a `SelectionSet` that always fails to decode.
///
/// This is handy when used with `SelectionSet::and_then` where you want to
/// give a custom error message in some case.
///
/// See the [`SelectionSet::and_then`](cynic::selection_set::SelectionSet::and_then)
/// docs for an example.
pub fn fail<'a, V>(err: impl Into<String>) -> SelectionSet<'static, V, ()> {
    SelectionSet {
        fields: vec![],
        decoder: json_decode::fail(err),
        phantom: PhantomData,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use assert_matches::assert_matches;

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
                    "String!",
                    required.required_arg,
                )];
                if optionals.opt_string.is_some() {
                    args.push(Argument::new(
                        "opt_string",
                        "String",
                        optionals.opt_string.unwrap(),
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

        let (query, args, _) = selection_set.query_arguments_and_decoder();

        assert_eq!(
            query,
            "test_struct {\n  field_one\n  nested {\n    a_string\n  }\n}\n"
        );
        assert!(args.is_empty());
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

        let (_query, args, _) = selection_set.query_arguments_and_decoder();
        assert_eq!(args.len(), 1);
    }

    #[test]
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

        let (_query, args, _) = selection_set.query_arguments_and_decoder();
        assert_eq!(args.len(), 2);
    }

    #[test]
    fn test_and_then() {
        let selection_set = string().and_then(|s| {
            if s == "ok" {
                succeed("YAS")
            } else {
                fail("no way min")
            }
        });

        assert_matches!(selection_set.decode(&serde_json::json!("ok")), Ok("YAS"));
        assert_matches!(selection_set.decode(&serde_json::json!("nope")), Err(_));
    }
}
