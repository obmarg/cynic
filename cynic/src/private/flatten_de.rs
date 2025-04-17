use std::marker::PhantomData;

use serde::Deserialize;

use crate::schema::OutputScalar;

pub struct Flattened<T> {
    inner: T,
}

impl<T> Flattened<T> {
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<'de, T> Deserialize<'de> for Flattened<Vec<T>>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Option::<Vec<Option<T>>>::deserialize(deserializer).map(|opt_vec| Flattened {
            inner: opt_vec
                .map(|vec| vec.into_iter().flatten().collect::<Vec<_>>())
                .unwrap_or_default(),
        })
    }
}

impl<'de, T> Deserialize<'de> for Flattened<Option<Vec<T>>>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Option::<Vec<Option<T>>>::deserialize(deserializer).map(|opt_vec| Flattened {
            inner: opt_vec.map(|vec| vec.into_iter().flatten().collect::<Vec<_>>()),
        })
    }
}

impl<'de, T, U> Deserialize<'de> for Flattened<super::ScalarDeserialize<Vec<T>, U>>
where
    Option<Vec<Option<T>>>: OutputScalar<'de, U>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        super::ScalarDeserialize::<Option<Vec<Option<T>>>, U>::deserialize(deserializer).map(
            |deser| Flattened {
                inner: super::ScalarDeserialize {
                    inner: deser
                        .into_inner()
                        .map(|vec| vec.into_iter().flatten().collect::<Vec<_>>())
                        .unwrap_or_default(),
                    phantom: PhantomData,
                },
            },
        )
    }
}

impl<'de, T, U> Deserialize<'de> for Flattened<super::ScalarDeserialize<Option<Vec<T>>, U>>
where
    Option<Vec<Option<T>>>: OutputScalar<'de, U>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        super::ScalarDeserialize::<Option<Vec<Option<T>>>, U>::deserialize(deserializer).map(
            |scalar_deser| Flattened {
                inner: scalar_deser.map(|opt_vec| {
                    opt_vec.map(|vec| vec.into_iter().flatten().collect::<Vec<_>>())
                }),
            },
        )
    }
}
