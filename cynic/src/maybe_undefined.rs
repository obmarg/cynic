use serde::{Deserialize, Serialize};

use std::ops::{Deref, DerefMut};

/// A wrapper around async-graphql's [`MaybeUndefined`](https://docs.rs/async-graphql/latest/async_graphql/types/enum.MaybeUndefined.html).
///
/// You can initialize it from:
/// - `From<Option<T>>` will become T or null.
/// - `Default` will become undefined.
#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct MaybeUndefined<T>(async_graphql::MaybeUndefined<T>);

impl<T> MaybeUndefined<T> {
    fn inner(&self) -> &async_graphql::MaybeUndefined<T> {
        &self.0
    }

    fn inner_mut(&mut self) -> &mut async_graphql::MaybeUndefined<T> {
        &mut self.0
    }

    /// Returns true if the `MaybeUndefined<T>` is undefined.
    ///
    /// Deserialization ought to be skipped for this when used as a field.
    pub fn is_undefined(&self) -> bool {
        self.0.is_undefined()
    }
}

impl<T> From<async_graphql::MaybeUndefined<T>> for MaybeUndefined<T> {
    fn from(value: async_graphql::MaybeUndefined<T>) -> Self {
        Self(value)
    }
}

impl<T> Deref for MaybeUndefined<T> {
    type Target = async_graphql::MaybeUndefined<T>;

    fn deref(&self) -> &Self::Target {
        self.inner()
    }
}

impl<T> DerefMut for MaybeUndefined<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner_mut()
    }
}

impl<T> From<Option<T>> for MaybeUndefined<T> {
    fn from(value: Option<T>) -> Self {
        Self(match value {
            Some(value) => async_graphql::MaybeUndefined::Value(value),
            None => async_graphql::MaybeUndefined::Null,
        })
    }
}

impl<T> Default for MaybeUndefined<T> {
    fn default() -> Self {
        Self(async_graphql::MaybeUndefined::Undefined)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        assert_eq!(
            MaybeUndefined::from(None),
            MaybeUndefined(async_graphql::MaybeUndefined::<bool>::Null)
        );
        assert_eq!(
            MaybeUndefined::from(Some(true)),
            MaybeUndefined(async_graphql::MaybeUndefined::Value(true))
        );
        assert_eq!(
            MaybeUndefined::<bool>::default(),
            MaybeUndefined(async_graphql::MaybeUndefined::Undefined)
        );
    }
}
