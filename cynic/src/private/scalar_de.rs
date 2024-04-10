pub struct ScalarDeseralize<T> {
    inner: T,
}

impl<T> ScalarDeseralize<T> {
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<'de, T, U> Deserialize<'de> for ScalarDeseralize<T>
where
    T: IsScalar<U>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        <T as IsScalar<U>>::deserialize(deserializer)
    }
}
