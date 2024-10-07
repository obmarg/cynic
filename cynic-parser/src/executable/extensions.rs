use std::{borrow::Cow, fmt};

use super::{
    generated::{name::Name, type_conditions::TypeCondition},
    FragmentDefinition, FragmentSpread,
};

impl<'a> FragmentSpread<'a> {
    pub fn fragment(&self) -> Option<FragmentDefinition<'a>> {
        let document = self.0.document;
        let needle = self.fragment_name();

        document
            .fragments()
            .find(|fragment| fragment.name() == needle)
    }
}

impl<'a> Name<'a> {
    pub fn as_str(&self) -> &'a str {
        self.text()
    }
}

impl PartialEq for Name<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl fmt::Display for Name<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text())
    }
}

impl<'a> From<Name<'a>> for Cow<'a, str> {
    fn from(value: Name<'a>) -> Self {
        Cow::Borrowed(value.text())
    }
}

impl<'a> From<TypeCondition<'a>> for Cow<'a, str> {
    fn from(value: TypeCondition<'a>) -> Self {
        Cow::Borrowed(value.on())
    }
}
