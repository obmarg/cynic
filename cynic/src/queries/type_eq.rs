/// Trait that determines whether two types are equal, that should
/// help improve cynics error messages.
///
/// Users should not impl this themselves. It's used in some
/// where bounds to help get better error messages in cynic.
pub trait IsFieldType<T> {}

impl<T> IsFieldType<T> for T {}
