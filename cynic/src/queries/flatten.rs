pub trait FlattensInto<T>: private::Sealed<T> {}

impl<T> FlattensInto<Vec<T>> for Vec<Option<T>> {}
impl<T> FlattensInto<Vec<T>> for Option<Vec<T>> {}
impl<T> FlattensInto<Option<Vec<T>>> for Option<Vec<Option<T>>> {}
impl<T> FlattensInto<Vec<T>> for Option<Vec<Option<T>>> {}

mod private {
    pub trait Sealed<T> {}

    impl<T> Sealed<Vec<T>> for Vec<Option<T>> {}
    impl<T> Sealed<Vec<T>> for Option<Vec<T>> {}
    impl<T> Sealed<Option<Vec<T>>> for Option<Vec<Option<T>>> {}
    impl<T> Sealed<Vec<T>> for Option<Vec<Option<T>>> {}
}
