/// Encodes the rules for whether a type can have the `#[cynic(recurse)]` attribute applied.
///
/// See [the relevant section of the book](https://cynic-rs.dev/derives/recursive-queries.html#recursive-field-types) for more details.
///
/// This type is sealed so users can't implement it.
pub trait Recursable<T>: private::Sealed {}

// Required fields get made optional to handle the base case.
impl<T> Recursable<T> for Option<T> {}

// Optional fields are recursable on their own.
impl<T> Recursable<Option<T>> for Option<T> {}

mod private {
    pub trait Sealed {}

    impl<T> Sealed for Option<T> {}
}
