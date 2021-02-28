use crate::{Id, SerializableArgument};

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
pub trait InputType<NamedType, Wrappers> {
    type Output: SerializableArgument;

    fn into_serializable(self) -> Self::Output;
}

impl<'a> InputType<String, NamedType> for &'a str {
    type Output = &'a str;

    fn into_serializable(self) -> &'a str {
        self
    }
}

impl<'a> InputType<String, Nullable<NamedType>> for &'a str {
    type Output = Option<&'a str>;

    fn into_serializable(self) -> Option<&'a str> {
        Some(self)
    }
}

impl<'a> InputType<String, Nullable<NamedType>> for Option<&'a str> {
    type Output = Option<&'a str>;

    fn into_serializable(self) -> Option<&'a str> {
        self
    }
}

/// Defines useful argument conversions for input objects
///
/// Mostly just converts references to owned via cloning and
/// non option-wrapped types into Option where appropriate.
#[macro_export]
macro_rules! impl_input_type {
    ($type:ty, $type_lock:path) => {
        impl $crate::InputType<$type_lock, $crate::inputs::NamedType> for $type {
            type Output = $type;

            fn into_serializable(self) -> Self::Output {
                self
            }
        }

        impl $crate::InputType<$type_lock, $crate::inputs::NamedType> for &$type {
            type Output = Self;

            fn into_serializable(self) -> Self::Output {
                self
            }
        }

        impl $crate::InputType<$type_lock, $crate::inputs::Nullable<$crate::inputs::NamedType>>
            for $type
        {
            type Output = Option<$type>;

            fn into_serializable(self) -> Option<$type> {
                Some(self)
            }
        }

        impl $crate::InputType<$type_lock, $crate::inputs::Nullable<$crate::inputs::NamedType>>
            for Option<$type>
        {
            type Output = Option<$type>;

            fn into_serializable(self) -> Option<$type> {
                self
            }
        }

        // TODO: See if _some_ of these can be made generic?
        // expect they can't but worth a shot..

        impl<'a> $crate::InputType<$type_lock, $crate::inputs::Nullable<$crate::inputs::NamedType>>
            for &'a $type
        {
            type Output = Option<&'a $type>;

            fn into_serializable(self) -> Option<&'a $type> {
                Some(self)
            }
        }

        impl<'a> $crate::InputType<$type_lock, $crate::inputs::Nullable<$crate::inputs::NamedType>>
            for Option<&'a $type>
        {
            type Output = Option<&'a $type>;

            fn into_serializable(self) -> Option<&'a $type> {
                self
            }
        }

        impl<'a> $crate::InputType<$type_lock, $crate::inputs::Nullable<$crate::inputs::NamedType>>
            for &'a Option<$type>
        {
            type Output = Option<&'a $type>;

            fn into_serializable(self) -> Option<&'a $type> {
                self.as_ref()
            }
        }

        impl
            $crate::InputType<
                $type_lock,
                $crate::inputs::Nullable<$crate::inputs::List<$crate::inputs::NamedType>>,
            > for Option<Vec<$type>>
        {
            type Output = Option<Vec<$type>>;

            fn into_serializable(self) -> Self::Output {
                self
            }
        }

        impl $crate::InputType<$type_lock, $crate::inputs::List<$crate::inputs::NamedType>>
            for Vec<$type>
        {
            type Output = Vec<$type>;

            fn into_serializable(self) -> Self::Output {
                self
            }
        }

        // TODO: Still feel there must be a way to do this.
        // Like by making Nullable<List<Nullable<Named>>> into a trait _somehow_
        // then using that to do most of the impls?
        // Not sure.
        // Like impl InputType<Lock, Wrappers> for T
        // where
        //   Lock: Something
        //   Wrappers: SomeOtherThing
        //   T: SomehowConstrainedForBoth
        impl
            $crate::InputType<
                $type_lock,
                $crate::inputs::Nullable<
                    $crate::inputs::List<$crate::inputs::Nullable<$crate::inputs::NamedType>>,
                >,
            > for Option<Vec<Option<$type>>>
        {
            type Output = Option<Vec<Option<$type>>>;

            fn into_serializable(self) -> Self::Output {
                self
            }
        }

        // TODO: Try and implement list coercion in here...
    };
}

impl_input_type!(i32, i32);
impl_input_type!(f64, f64);
impl_input_type!(String, String);
impl_input_type!(bool, bool);
impl_input_type!(Id, Id);
