use crate::SpecificationVersion;

/// The set of capaiblities a GraphQL server supports.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct CapabilitySet {
    pub(super) specification_version: SpecificationVersion,
}

impl CapabilitySet {
    /// The version of the GraphQL specification this server supports
    pub fn version_supported(&self) -> SpecificationVersion {
        self.specification_version
    }
}

impl SpecificationVersion {
    /// The capabilities of a server that implements this version of the specification
    pub fn capabilities(self) -> CapabilitySet {
        CapabilitySet {
            specification_version: self,
        }
    }
}
