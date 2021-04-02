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

/// Defines common `InputType` impls for a given type & type lock.
///
/// The `InputType` trait is used to allow conversions while enforcing type
/// safety on the various input types (scalars, input objects & enums)
/// when they appear in input position.  For example an optional field
/// will have an InputType definition that allows for non `Option` values
/// to be passed.
///
/// These impls can't be defined generically as there's a lot of them and
/// we quickly run into clashing impls.  This macro defines specific impls
/// for each type, working around this.
///
/// Users usually shouldn't need to call this, as it's called by other macros.
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
