use std::{borrow::Cow, marker::PhantomData};

use serde::de::Visitor;

pub struct KeyDeserializer<'de, E> {
    key: Cow<'de, str>,
    phantom: PhantomData<fn() -> E>,
}

impl<'de, E> KeyDeserializer<'de, E> {
    pub fn new(key: Cow<'de, str>) -> Self {
        KeyDeserializer {
            key,
            phantom: PhantomData,
        }
    }
}

impl<'de, E> serde::de::IntoDeserializer<'de, E> for KeyDeserializer<'de, E>
where
    E: serde::de::Error,
{
    type Deserializer = Self;

    fn into_deserializer(self) -> Self {
        self
    }
}

impl<'de, E> serde::de::Deserializer<'de> for KeyDeserializer<'de, E>
where
    E: serde::de::Error,
{
    type Error = E;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.key {
            Cow::Borrowed(x) => visitor.visit_borrowed_str(x),
            Cow::Owned(s) => visitor.visit_string(s),
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}
