use std::{borrow::Cow, marker::PhantomData};

use serde::de::{Error, MapAccess, Visitor};

use super::{
    content::{Content, ContentDeserializer},
    cow_str::CowStr,
    key_de::KeyDeserializer,
};

pub struct InlineFragmentVisitor<T> {
    phantom: PhantomData<fn() -> T>,
}

impl<T> InlineFragmentVisitor<T> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        InlineFragmentVisitor {
            phantom: PhantomData,
        }
    }
}

impl<'de, T> Visitor<'de> for InlineFragmentVisitor<T>
where
    T: crate::core::InlineFragments<'de>,
{
    type Value = T;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a map")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut buffer = Vec::new();

        while let Some(key) = access.next_key::<CowStr>()? {
            let key = key.into_inner();
            if key == "__typename" {
                let typename = access.next_value::<CowStr>()?.into_inner();
                return T::deserialize_variant(
                    typename.as_ref(),
                    BufferDeserializer { access, buffer },
                );
            }
            buffer.push((key, access.next_value::<Content>()?))
        }

        Err(M::Error::missing_field("__typename"))
    }
}

struct BufferDeserializer<'de, M: MapAccess<'de>> {
    access: M,
    buffer: Vec<(Cow<'de, str>, Content<'de>)>,
}

impl<'de, M> serde::de::Deserializer<'de> for BufferDeserializer<'de, M>
where
    M: MapAccess<'de>,
{
    type Error = M::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(BufferMapAccess {
            access: self.access,
            buffer: self.buffer,
            next_content: None,
        })
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct BufferMapAccess<'de, M: MapAccess<'de>> {
    access: M,
    buffer: Vec<(Cow<'de, str>, Content<'de>)>,
    next_content: Option<Content<'de>>,
}

impl<'de, M> serde::de::MapAccess<'de> for BufferMapAccess<'de, M>
where
    M: MapAccess<'de>,
{
    type Error = M::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        if let Some((key, content)) = self.buffer.pop() {
            self.next_content = Some(content);
            return seed.deserialize(KeyDeserializer::new(key)).map(Some);
        }

        self.access.next_key_seed(seed)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        if let Some(content) = self.next_content.take() {
            return seed.deserialize(ContentDeserializer::new(content));
        }

        self.access.next_value_seed(seed)
    }
}
