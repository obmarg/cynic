use cynic::{Operation, OperationBuilder};

use crate::{CapabilitySet, IntrospectionQuery, SpecificationVersion};

impl IntrospectionQuery {
    /// Builds an IntrospectionQuery for a server with the given capabilities
    pub fn with_capabilities(capabilities: CapabilitySet) -> Operation<Self, ()> {
        let mut builder = OperationBuilder::query().with_variables(());
        match capabilities.specification_version {
            SpecificationVersion::Unknown | SpecificationVersion::June2018 => {}
            SpecificationVersion::October2021 => builder.enable_feature("2021"),
            SpecificationVersion::September2025 => {
                builder.enable_feature("2021");
                builder.enable_feature("2025");
            }
        }
        builder.build().expect("to succeed")
    }
}
