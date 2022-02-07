use std::marker::PhantomData;

use serde::{de::Visitor, Deserialize};

pub struct Flattened<T> {
    inner: T,
}

impl<T> Flattened<T> {
    pub fn into_inner(self) -> T {
        self.inner
    }
}

// impl<'de, T> Deserialize<'de> for Flattened<Option<T>>
// where
//     T: Deserialize<'de>,
//     Option<Flattened<T>>: Deserialize<'de>,
// {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//     }
// }

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
