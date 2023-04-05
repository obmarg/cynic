use serde::{Deserialize, Serialize};

/// A GraphQL `ID`
///
/// Any field in a GraphQL schema that has the type `ID` should be represented
/// by this struct.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, ref_cast::RefCast)]
#[repr(transparent)]
pub struct Id(String);

impl Id {
    /// Constructs an `ID` from a `String`, `&str` or similar
    ///
    /// ```
    /// cynic::Id::new("123");
    /// ```
    pub fn new(s: impl Into<String>) -> Self {
        Id(s.into())
    }

    /// Returns a reference to the value of this `Id`
    pub fn inner(&self) -> &str {
        &self.0
    }

    /// Converts this `Id` into its inner value
    pub fn into_inner(self) -> String {
        self.0
    }

    /// Converts a reference to a String to a reference to an Id
    ///
    /// To be used when you can access an `&String` which you want to assume is
    /// an `Id` for use in Cynic structures without reallocating
    ///
    /// If you don't have a `String` at hand but only an `&str`, you should know
    /// that these can be used directly in `InputObject`s as well when the
    /// target GraphQL type is an `Id`.
    pub fn from_ref(s: &String) -> &Self {
        // Unfortunately this won't work with `&str`

        // Probably the best design if we really wanted to enforce more typing around
        // Id would be to have `IdSlice { inner: str }` for that case,
        // that has ref-casting enabled as well, and to have `Id: Deref<Target =
        // IdSlice>`.

        // However:
        // - most likely you'll have an actual `&String` at hand when doing that and not
        //   only an `&str`.
        // - If you don't it's probably acceptable to either use an `&str` in the
        //   `InputObject` (which you can because of `CoercesTo` impl) or allocate
        // - That would significantly add to the complexity of the library for such a
        //   niche use-case
        // - To be consistent with enforcing more typing we'd probably want to remove
        //   `CoercesTo<Id> for str` and rename the `new` function on Id (and IdSlice)
        //   to `assume_exists`, which would be more boilerplateish and non
        //   backwards-compatible

        // So overall it's probably not worth
        ref_cast::RefCast::ref_cast(s)
    }
}

impl<T: Into<String>> From<T> for Id {
    fn from(s: T) -> Id {
        Id(s.into())
    }
}

impl<'a> From<&'a String> for &'a Id {
    fn from(s: &'a String) -> Self {
        Id::from_ref(s)
    }
}
