//! GraphQL has a number of coercion rules that make it easier to use and allow
//! certain changes to be made in a backwards compatible way.
//!
//! This module contains some traits & macros that are used to implement these
//! rules in cynic.

use crate::Id;

/// Determines whether a type can be coerced into a given schema type.
///
/// Users should not usually need to implement this, it's handled automatically
/// by the various derives.
pub trait CoercesTo<T> {}

impl<T, TypeLock> CoercesTo<Option<TypeLock>> for Option<T> where T: CoercesTo<TypeLock> {}
impl<T, TypeLock> CoercesTo<Vec<TypeLock>> for Vec<T> where T: CoercesTo<TypeLock> {}

impl CoercesTo<Id> for &str {}

#[macro_export]
/// Implements the default GraphQL list & option coercion rules for a type.
macro_rules! impl_coercions {
    ($target:ty, $typelock:ty) => {
        impl $crate::coercions::CoercesTo<$typelock> for $target {}
        impl $crate::coercions::CoercesTo<Option<$typelock>> for $target {}
        impl $crate::coercions::CoercesTo<Vec<$typelock>> for $target {}
        impl $crate::coercions::CoercesTo<Option<Vec<$typelock>>> for $target {}
        impl $crate::coercions::CoercesTo<Option<Vec<Option<$typelock>>>> for $target {}
        impl $crate::coercions::CoercesTo<Option<Option<$typelock>>> for $target {}
        impl $crate::coercions::CoercesTo<Vec<Vec<$typelock>>> for $target {}
    };
}

mod scalars {
    use super::*;

    macro_rules! impl_coercions_for_scalar {
        ($target:ty) => {
            crate::impl_coercions!($target, $target);
        };
    }

    impl_coercions_for_scalar!(i32);
    impl_coercions_for_scalar!(f64);
    impl_coercions_for_scalar!(bool);
    impl_coercions_for_scalar!(Id);
    impl_coercions_for_scalar!(String);
    impl_coercions!(&str, String);
}

#[cfg(test)]
mod tests {
    use static_assertions::{assert_impl_all, assert_not_impl_any};

    use super::*;

    #[test]
    fn test_coercions() {
        assert_impl_all!(i32: CoercesTo<i32>);
        assert_impl_all!(i32: CoercesTo<Option<i32>>);
        assert_impl_all!(i32: CoercesTo<Vec<i32>>);
        assert_impl_all!(i32: CoercesTo<Option<Vec<i32>>>);
        assert_impl_all!(i32: CoercesTo<Vec<Vec<i32>>>);

        assert_impl_all!(Option<i32>: CoercesTo<Option<i32>>);
        assert_impl_all!(Option<i32>: CoercesTo<Option<Option<i32>>>);

        assert_impl_all!(Vec<i32>: CoercesTo<Vec<i32>>);
        assert_impl_all!(Vec<i32>: CoercesTo<Vec<Vec<i32>>>);

        assert_not_impl_any!(Vec<i32>: CoercesTo<i32>);
        assert_not_impl_any!(Option<i32>: CoercesTo<i32>);
    }
}
