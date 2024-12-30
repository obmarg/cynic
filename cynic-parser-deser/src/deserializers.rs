use crate::{deserialize::ValueDeserialize, value::Object, DeserValue, Error};

pub trait ConstDeserializer<'a> {
    fn deserialize<T: ValueDeserialize<'a>>(self) -> Result<T, Error>;
}

impl<'a> ConstDeserializer<'a> for cynic_parser::ConstValue<'a> {
    fn deserialize<T>(self) -> Result<T, Error>
    where
        T: ValueDeserialize<'a>,
    {
        T::deserialize(DeserValue::from_const(self))
    }
}

impl<'a> ConstDeserializer<'a> for DeserValue<'a> {
    fn deserialize<T>(self) -> Result<T, Error>
    where
        T: ValueDeserialize<'a>,
    {
        T::deserialize(self)
    }
}

impl<'a> ConstDeserializer<'a> for cynic_parser::type_system::Directive<'a> {
    fn deserialize<T>(self) -> Result<T, Error>
    where
        T: ValueDeserialize<'a>,
    {
        T::deserialize(DeserValue::Object(Object::from_type_system_directive(self)))
    }
}
