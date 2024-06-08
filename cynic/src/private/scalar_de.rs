use std::marker::PhantomData;

use serde::Deserialize;

use crate::schema::{IsOutputScalar, IsScalar};

pub struct ScalarDeseralize<T, U> {
    inner: T,
    phantom: PhantomData<fn() -> U>,
}

impl<T, U> ScalarDeseralize<T, U> {
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<'de, T, U> Deserialize<'de> for ScalarDeseralize<T, U>
where
    T: IsOutputScalar<U>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let inner = <T as IsOutputScalar<U>>::deserialize(deserializer)?;

        Ok(ScalarDeseralize {
            inner,
            phantom: PhantomData,
        })
    }
}
