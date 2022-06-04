use std::borrow::Cow;

use serde::{
    ser::{self, Error as _},
    Serialize,
};

use crate::queries::{Argument, InputLiteral};

/// Serializes a type into an `InputLiteral`
pub fn to_input_literal<T>(value: &T) -> Result<InputLiteral, Error>
where
    T: Serialize + ?Sized,
{
    value.serialize(InputLiteralSerializer)
}

struct InputLiteralSerializer;

impl<'a> ser::Serializer for InputLiteralSerializer {
    type Ok = InputLiteral;

    type Error = Error;

    type SerializeSeq = ListSerializer;

    type SerializeTuple = ListSerializer;

    type SerializeTupleStruct = ser::Impossible<InputLiteral, Error>;

    type SerializeTupleVariant = ser::Impossible<InputLiteral, Error>;

    type SerializeMap = MapSerializer;

    type SerializeStruct = MapSerializer;

    type SerializeStructVariant = ser::Impossible<InputLiteral, Error>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(InputLiteral::Bool(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(InputLiteral::Int(v as i32))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(InputLiteral::Int(v as i32))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(InputLiteral::Int(v))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(InputLiteral::Int(i32::try_from(v).map_err(|e| {
            Error::custom(format!("graphql integers must fit into an i32: {e}"))
        })?))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(InputLiteral::Int(v as i32))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(InputLiteral::Int(v as i32))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(InputLiteral::Int(i32::try_from(v).map_err(|e| {
            Error::custom(format!("graphql integers must fit into an i32: {e}"))
        })?))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(InputLiteral::Int(i32::try_from(v).map_err(|e| {
            Error::custom(format!("graphql integers must fit into an i32: {e}"))
        })?))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(InputLiteral::Float(v as f64))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(InputLiteral::Float(v))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(InputLiteral::String(v.to_string().into()))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(InputLiteral::String(v.to_string().into()))
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom("cannot serialize bytes as an InputLiteral"))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(InputLiteral::Null)
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom("cannot serialize unit as an InputLiteral"))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(
            "cannot serialize a unit struct as an InputLiteral",
        ))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(InputLiteral::EnumValue(variant))
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(Error::custom(
            "cannot serialize a newtype variant as an InputLiteral",
        ))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let items = match len {
            Some(len) => Vec::with_capacity(len),
            None => Vec::new(),
        };

        Ok(ListSerializer { items })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(Error::custom(
            "cannot serialize a tuple struct as an InputLiteral",
        ))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Error::custom(
            "cannot serialize a tuple variant as an InputLiteral",
        ))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        let entries = match len {
            Some(len) => Vec::with_capacity(len),
            None => Vec::new(),
        };

        Ok(MapSerializer {
            entries,
            next_key: None,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(Error::custom(
            "cannot serialize a struct variant as an InputLiteral",
        ))
    }
}

struct ListSerializer {
    items: Vec<InputLiteral>,
}

impl<'a> ser::SerializeSeq for ListSerializer {
    type Ok = InputLiteral;

    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.items.push(value.serialize(InputLiteralSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(InputLiteral::List(self.items))
    }
}

impl<'a> ser::SerializeTuple for ListSerializer {
    type Ok = InputLiteral;

    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.items.push(to_input_literal(value)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(InputLiteral::List(self.items))
    }
}

struct MapSerializer {
    entries: Vec<Argument>,
    next_key: Option<Cow<'static, str>>,
}

impl<'a> ser::SerializeMap for MapSerializer {
    type Ok = InputLiteral;

    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.next_key = Some(key.serialize(KeySerializer)?);
        Ok(())
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.entries.push(Argument::from_cow_name(
            self.next_key.take().unwrap(),
            to_input_literal(value)?,
        ));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(InputLiteral::Object(self.entries))
    }
}

impl<'a> ser::SerializeStruct for MapSerializer {
    type Ok = InputLiteral;

    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        self.entries
            .push(Argument::new(key, to_input_literal(value)?));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(InputLiteral::Object(self.entries))
    }
}

struct KeySerializer;

impl ser::Serializer for KeySerializer {
    type Ok = Cow<'static, str>;

    type Error = Error;

    type SerializeSeq = ser::Impossible<Cow<'static, str>, Error>;

    type SerializeTuple = ser::Impossible<Cow<'static, str>, Error>;

    type SerializeTupleStruct = ser::Impossible<Cow<'static, str>, Error>;

    type SerializeTupleVariant = ser::Impossible<Cow<'static, str>, Error>;

    type SerializeMap = ser::Impossible<Cow<'static, str>, Error>;

    type SerializeStruct = ser::Impossible<Cow<'static, str>, Error>;

    type SerializeStructVariant = ser::Impossible<Cow<'static, str>, Error>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string().into())
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(
            "Map keys must be serializable as strings, found an int",
        ))
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(
            "Map keys must be serializable as strings found an int",
        ))
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(
            "Map keys must be serializable as strings found an int",
        ))
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(
            "Map keys must be serializable as strings found an int",
        ))
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(
            "Map keys must be serializable as strings found an int",
        ))
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(
            "Map keys must be serializable as strings found an int",
        ))
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(
            "Map keys must be serializable as strings found an int",
        ))
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(
            "Map keys must be serializable as strings found an int",
        ))
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(
            "Map keys must be serializable as strings found a float",
        ))
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(
            "Map keys must be serializable as strings found a float",
        ))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string().into())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(v.to_string().into())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(
            "Map keys must be serializable as strings found some bytes",
        ))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(
            "Map keys must be serializable as strings found null",
        ))
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(
            "Map keys must be serializable as strings found null",
        ))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(Error::custom(
            "Map keys must be serializable as strings found a struct",
        ))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(Cow::Borrowed(variant))
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(Error::custom(
            "Map keys must be serializable as strings found a struct",
        ))
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        Err(Error::custom(
            "Map keys must be serializable as strings found a newtype variant",
        ))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(Error::custom(
            "Map keys must be serializable as strings found a sequence",
        ))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(Error::custom(
            "Map keys must be serializable as strings found a sequence",
        ))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(Error::custom(
            "Map keys must be serializable as strings found a tuple",
        ))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Error::custom(
            "Map keys must be serializable as strings found a tuple",
        ))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(Error::custom(
            "Map keys must be serializable as strings found a map",
        ))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(Error::custom(
            "Map keys must be serializable as strings found a struct",
        ))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(Error::custom(
            "Map keys must be serializable as strings found a struct",
        ))
    }
}

#[derive(thiserror::Error, Debug)]
#[error("could not serialize to InputLiteral: {0}")]
pub struct Error(String);

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Error(msg.to_string())
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(16, InputLiteral::Int(16))]
    #[case(1.5, InputLiteral::Float(1.5))]
    #[case(true, InputLiteral::Bool(true))]
    #[case("hello".to_string(), InputLiteral::String("hello".to_string().into()))]
    #[case(
        maplit::btreemap! {
            "bar" => 20,
            "foo" => 16
        },
        InputLiteral::Object(vec![
            Argument::new("bar", InputLiteral::Int(20)),
            Argument::new("foo", InputLiteral::Int(16))
        ])
    )]
    #[case(
        (1, 2, 3),
        InputLiteral::List(vec![
            InputLiteral::Int(1),
            InputLiteral::Int(2),
            InputLiteral::Int(3)
        ])
    )]
    #[case(
        vec![1, 2, 3],
        InputLiteral::List(vec![
            InputLiteral::Int(1),
            InputLiteral::Int(2),
            InputLiteral::Int(3)
        ])
    )]
    fn test_serialization(#[case] input: impl serde::Serialize, #[case] expected: InputLiteral) {
        assert_eq!(to_input_literal(&input).unwrap(), expected)
    }
}
