/// An integer value
///
/// The GraphQl Int scalar is an i32, however the GraphQl language doesn't
/// impose specific limits on integers.  Currently cynic-parser represents
/// an IntValue as an i64, but we hide this fact behind a newtype to allow
/// us to change the internal representaiton later if we need to.
#[derive(Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct IntValue(pub(crate) i64);

impl IntValue {
    pub fn as_i64(&self) -> i64 {
        self.0
    }

    pub fn as_i32(&self) -> i32 {
        self.0 as i32
    }
}

impl std::fmt::Debug for IntValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for IntValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
