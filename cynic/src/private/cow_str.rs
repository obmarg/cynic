use std::{borrow::Cow, fmt};

use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer,
};

/// Wrapper around a Cow to allow for custom deserialization.
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct CowStr<'de>(Cow<'de, str>);

impl<'de> CowStr<'de> {
    pub fn into_inner(self) -> Cow<'de, str> {
        self.0
    }
}

impl<'de> Deserialize<'de> for CowStr<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(CowStrVisitor)
    }
}

struct CowStrVisitor;

impl<'de> Visitor<'de> for CowStrVisitor {
    type Value = CowStr<'de>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a string")
    }

    // Borrowed directly from the input string, which has lifetime 'de
    // The input must outlive the resulting Cow.
    fn visit_borrowed_str<E>(self, value: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(CowStr(Cow::Borrowed(value)))
    }

    // A string that currently only lives in a temporary buffer -- we need a copy
    // (Example: serde is reading from a BufRead)
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(CowStr(Cow::Owned(value.to_owned())))
    }

    // An optimisation of visit_str for situations where the deserializer has
    // already taken ownership. For example, the string contains escaped characters.
    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(CowStr(Cow::Owned(value)))
    }
}
