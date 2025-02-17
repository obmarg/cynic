use std::marker::PhantomData;

use serde::Deserialize;

use crate::schema::{IsOutputScalar, IsScalar};

pub struct ScalarDeserialize<T, U> {
    pub(super) inner: T,
    pub(super) phantom: PhantomData<fn() -> U>,
}

impl<T, U> ScalarDeserialize<T, U> {
    pub fn into_inner(self) -> T {
        self.inner
    }

    pub fn map<NewT, F>(self, fun: F) -> ScalarDeserialize<NewT, U>
    where
        F: FnOnce(T) -> NewT,
    {
        ScalarDeserialize {
            inner: fun(self.inner),
            phantom: PhantomData,
        }
    }
}

impl<'de, T, U> Deserialize<'de> for ScalarDeserialize<T, U>
where
    T: IsOutputScalar<'de, U>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let inner = <T as IsOutputScalar<U>>::deserialize(deserializer)?;

        Ok(ScalarDeserialize {
            inner,
            phantom: PhantomData,
        })
    }
}

pub struct ScalarSerialize<'a, T, U> {
    inner: &'a T,
    phantom: PhantomData<fn(U) -> ()>,
}

impl<'a, T, U> ScalarSerialize<'a, T, U> {
    pub fn new(inner: &'a T) -> Self {
        ScalarSerialize {
            inner,
            phantom: PhantomData,
        }
    }
}

impl<T, U> serde::Serialize for ScalarSerialize<'_, T, U>
where
    T: IsScalar<U>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        IsScalar::serialize(self.inner, serializer)
    }
}
