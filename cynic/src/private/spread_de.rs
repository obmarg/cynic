use std::{collections::HashMap, marker::PhantomData, rc::Rc};

use serde::de::{self, Deserialize, Deserializer, MapAccess};

use super::{
    content::{Content, ContentRefDeserializer},
    key_de::KeyDeserializer,
};

pub struct Spreadable<'de, E> {
    fields: HashMap<&'de str, Content<'de>>,
    error: PhantomData<fn() -> E>,
}

impl<'de, E> Deserialize<'de> for Spreadable<'de, E> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let fields = HashMap::deserialize(deserializer)?;
        Ok(Spreadable {
            fields,
            error: PhantomData,
        })
    }
}

impl<'de, E> Spreadable<'de, E>
where
    E: serde::de::Error,
{
    pub fn deserialize_field<T>(&self, field: &'static str) -> Result<T, E>
    where
        T: serde::de::Deserialize<'de>,
    {
        if let Some(content) = self.fields.get(field) {
            return T::deserialize(ContentRefDeserializer::new(content));
        }

        Err(E::missing_field(field))
    }

    pub fn spread_deserializer(&'_ self) -> impl Deserializer<'de> + '_ {
        SpreadDeserializer::<E> {
            iter: self.fields.iter(),
            next_content: None,
            error: PhantomData,
        }
    }
}

struct SpreadDeserializer<'a, 'de, E> {
    iter: std::collections::hash_map::Iter<'a, &'de str, Content<'de>>,
    next_content: Option<&'a Content<'de>>,
    error: PhantomData<fn() -> E>,
}

impl<'a, 'de, E> Deserializer<'de> for SpreadDeserializer<'a, 'de, E>
where
    E: de::Error,
{
    type Error = E;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'a, 'de, E> MapAccess<'de> for SpreadDeserializer<'a, 'de, E>
where
    E: de::Error,
{
    type Error = E;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if let Some((key, content)) = self.iter.next() {
            self.next_content = Some(content);
            return seed.deserialize(KeyDeserializer::new(key)).map(Some);
        }

        Ok(None)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let content = self
            .next_content
            .take()
            .expect("next_value_seed called before next_key_seed");

        seed.deserialize(ContentRefDeserializer::new(content))
    }
}
