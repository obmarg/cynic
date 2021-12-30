//! GraphQL has a lot of input coercion rules.
//!
//! This module provides a trait `InputType` that helps implement these (as far as possible)
//! in rust.  Users shouldn't usually need to care about the details of this - the cynic
//! derives output most implementations of these traits that are required.

/// Marker type for a named `InputType`
///
/// Users usually shouldn't need to worry about this - it's used as the inner type of the
/// `Wrappers` type parameter of the `InputType` trait.  e.g. a `Wrapper` of `NamedType` indicates
/// a required input, a `Wrapper` of `Nullable<NamedType>` indicates an optional input etc.
pub enum NamedType {}

/// Marker type for a nullable `InputType`
///
/// Users usually shouldn't need to worry about this - it's used as a wrapper for the
/// `Wrappers` type parameter of the `InputType` trait.
///
/// e.g. a `Wrapper` of `Nullable<NamedType>` indicates an optional input.
pub struct Nullable<T>(std::marker::PhantomData<T>);

/// Marker type for a nullable `InputType`
///
/// Users usually shouldn't need to worry about this - it's used as a wrapper for the
/// `Wrappers` type parameter of the `InputType` trait.
///
/// e.g. a `Wrapper` of `List<Nullable<NamedType>>` indicates a list of nullable inputs
pub struct List<T>(std::marker::PhantomData<T>);

/// A trait for accepting input types.  This is used to determine whether a type is compatible
/// with the expected input type when provided as an argument: either directly as an field
/// argument or inside an InputObject.
///
/// This trait has two type parameters:
///
/// - `NamedType` should point to the marker trait of the underlying type in your
///   `schema` module.
/// - `Wrappers` is used to specify the "wrapper types", for example if it is nullable
///    or in a list.
pub trait InputType<NamedType, Wrappers>: serde::Serialize {
    //fn as_serializable(&self) -> Self::Output;
    //fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>;
    fn into_upload(&self) -> Option<&crate::Upload> {
        None
    }
}

impl<T: ?Sized, TypeLock, Wrappers> InputType<TypeLock, Wrappers> for &T
where
    T: InputType<TypeLock, Wrappers>,
{
    fn into_upload(&self) -> Option<&crate::Upload> {
        (*self).into_upload()
    }
}

impl<T: ?Sized, TypeLock, Wrappers> InputType<TypeLock, Wrappers> for Box<T>
where
    T: InputType<TypeLock, Wrappers>,
{
    fn into_upload(&self) -> Option<&crate::Upload> {
        (*self.as_ref()).into_upload()
    }
}

impl<T: ?Sized, TypeLock, Wrappers> InputType<TypeLock, Wrappers> for std::rc::Rc<T>
where
    T: InputType<TypeLock, Wrappers>,
    std::rc::Rc<T>: serde::Serialize,
{
    fn into_upload(&self) -> Option<&crate::Upload> {
        (*self.as_ref()).into_upload()
    }
}

impl<T: ?Sized, TypeLock, Wrappers> InputType<TypeLock, Wrappers> for std::sync::Arc<T>
where
    T: InputType<TypeLock, Wrappers>,
    std::sync::Arc<T>: serde::Serialize,
{
    fn into_upload(&self) -> Option<&crate::Upload> {
        (*self.as_ref()).into_upload()
    }
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
        $crate::impl_input_type! {$type, $type_lock, |_me| None}
    };
    ($type:ty, $type_lock:path, $converter:expr) => {
        impl $crate::InputType<$type_lock, $crate::inputs::NamedType> for $type {
            fn into_upload(&self) -> Option<&$crate::Upload> {
                $converter(self)
            }
        }

        impl $crate::InputType<$type_lock, $crate::inputs::Nullable<$crate::inputs::NamedType>>
            for $type
        {
            fn into_upload(&self) -> Option<&$crate::Upload> {
                $converter(self)
            }
        }

        impl $crate::InputType<$type_lock, $crate::inputs::Nullable<$crate::inputs::NamedType>>
            for Option<$type>
        {
            fn into_upload(&self) -> Option<&$crate::Upload> {
                self.as_ref().and_then($converter)
            }
        }

        impl<'a> $crate::InputType<$type_lock, $crate::inputs::Nullable<$crate::inputs::NamedType>>
            for Option<&'a $type>
        {
            fn into_upload(&self) -> Option<&$crate::Upload> {
                self.and_then($converter)
            }
        }

        impl
            $crate::InputType<
                $type_lock,
                $crate::inputs::Nullable<$crate::inputs::List<$crate::inputs::NamedType>>,
            > for Option<Vec<$type>>
        {
            fn into_upload(&self) -> Option<&$crate::Upload> {
                None
            }
        }

        impl $crate::InputType<$type_lock, $crate::inputs::List<$crate::inputs::NamedType>>
            for Vec<$type>
        {
            fn into_upload(&self) -> Option<&$crate::Upload> {
                None
            }
        }

        impl
            $crate::InputType<
                $type_lock,
                $crate::inputs::Nullable<
                    $crate::inputs::List<$crate::inputs::Nullable<$crate::inputs::NamedType>>,
                >,
            > for Option<Vec<Option<$type>>>
        {
            fn into_upload(&self) -> Option<&$crate::Upload> {
                None
            }
        }
    };
}
