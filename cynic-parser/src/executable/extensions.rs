use super::{FragmentDefinition, FragmentSpread};

impl<'a> FragmentSpread<'a> {
    pub fn fragment(&self) -> Option<FragmentDefinition<'a>> {
        let document = self.0.document;
        let needle = self.fragment_name();

        document
            .fragments()
            .find(|fragment| fragment.name() == needle)
    }
}
