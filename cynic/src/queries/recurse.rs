pub trait Recursable<T> {}

// Required fields get made optional to handle the base case.
impl<T> Recursable<T> for Option<T> {}

// Optional fields are recursable on their own.
impl<T> Recursable<Option<T>> for Option<T> {}

// // Required lists get wrapped in option to handle the base case.
// impl<T> Recursable<Vec<T>> for Option<Vec<T>> {}

// // Optional lists need no transformation
// impl<T> Recursable<Option<Vec<T>>> for Option<Vec<T>> {}
