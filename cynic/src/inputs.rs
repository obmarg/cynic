pub enum NamedType {}

pub struct Nullable<T>(std::marker::PhantomData<T>);
pub struct List<T>(std::marker::PhantomData<T>);

/// A trait for accepting input types.  This is used to determine whether a type is compatible
/// with the expected input type when provided as an argument: either directly as an field
/// argument or inside an InputObject.
///
/// This trait has two type parameters:
///
/// - `NamedType` should point to the marker trait of the underlying type in your
///   `query_dsl` module.
/// - `Wrappers` is used to specify the "wrapper types", for example if it is nullable
///    or in a list.
pub trait InputType<NamedType, Wrappers>: serde::Serialize {
    //fn as_serializable(&self) -> Self::Output;
    //fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>;
}

impl<T: ?Sized, TypeLock, Wrappers> InputType<TypeLock, Wrappers> for &T where
    T: InputType<TypeLock, Wrappers>
{
}

impl<T: ?Sized, TypeLock, Wrappers> InputType<TypeLock, Wrappers> for Box<T> where
    T: InputType<TypeLock, Wrappers>
{
}

impl<T: ?Sized, TypeLock, Wrappers> InputType<TypeLock, Wrappers> for std::rc::Rc<T>
where
    T: InputType<TypeLock, Wrappers>,
    std::rc::Rc<T>: serde::Serialize,
{
}

impl<T: ?Sized, TypeLock, Wrappers> InputType<TypeLock, Wrappers> for std::sync::Arc<T>
where
    T: InputType<TypeLock, Wrappers>,
    std::sync::Arc<T>: serde::Serialize,
{
}

impl<'a> InputType<String, NamedType> for &'a str {}

impl<'a> InputType<String, Nullable<NamedType>> for &'a str {}

impl<'a> InputType<String, Nullable<NamedType>> for Option<&'a str> {}

/// Defines useful argument conversions for input objects
///
/// Mostly just converts references to owned via cloning and
/// non option-wrapped types into Option where appropriate.
#[macro_export]
macro_rules! impl_input_type {
    ($type:ty, $type_lock:path) => {
        impl $crate::InputType<$type_lock, $crate::inputs::NamedType> for $type {}

        impl $crate::InputType<$type_lock, $crate::inputs::Nullable<$crate::inputs::NamedType>>
            for $type
        {
        }

        impl $crate::InputType<$type_lock, $crate::inputs::Nullable<$crate::inputs::NamedType>>
            for Option<$type>
        {
        }

        impl<'a> $crate::InputType<$type_lock, $crate::inputs::Nullable<$crate::inputs::NamedType>>
            for Option<&'a $type>
        {
        }

        impl
            $crate::InputType<
                $type_lock,
                $crate::inputs::Nullable<$crate::inputs::List<$crate::inputs::NamedType>>,
            > for Option<Vec<$type>>
        {
        }

        impl $crate::InputType<$type_lock, $crate::inputs::List<$crate::inputs::NamedType>>
            for Vec<$type>
        {
        }

        impl
            $crate::InputType<
                $type_lock,
                $crate::inputs::Nullable<
                    $crate::inputs::List<$crate::inputs::Nullable<$crate::inputs::NamedType>>,
                >,
            > for Option<Vec<Option<$type>>>
        {
        }
    };
}
