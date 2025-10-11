use std::{borrow::Cow, fmt};

use crate::{common::OperationType, type_system::InputObjectDefinition};

use super::{Argument, Description, Directive, RootOperationTypeDefinition, SchemaDefinition};

impl<'a> Description<'a> {
    pub fn to_cow(&self) -> Cow<'a, str> {
        self.literal().to_cow()
    }

    pub fn raw_untrimmed_str(&self) -> &'a str {
        self.literal().raw_untrimmed_str()
    }
}

impl fmt::Display for Description<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.literal().fmt(f)
    }
}

impl<'a> SchemaDefinition<'a> {
    pub fn root_definition(
        &self,
        operation_type: OperationType,
    ) -> Option<RootOperationTypeDefinition<'a>> {
        self.root_operations()
            .find(|op| op.operation_type() == operation_type)
    }

    pub fn root_query_definition(&self) -> Option<RootOperationTypeDefinition<'a>> {
        self.root_definition(OperationType::Query)
    }

    pub fn root_mutation_definition(&self) -> Option<RootOperationTypeDefinition<'a>> {
        self.root_definition(OperationType::Mutation)
    }

    pub fn root_subscription_definition(&self) -> Option<RootOperationTypeDefinition<'a>> {
        self.root_definition(OperationType::Subscription)
    }
}

impl<'a> Directive<'a> {
    pub fn argument(&self, name: &str) -> Option<Argument<'a>> {
        self.arguments().find(|arg| arg.name() == name)
    }
}

impl InputObjectDefinition<'_> {
    pub fn is_one_of(&self) -> bool {
        self.directives()
            .any(|directive| directive.name() == "oneOf")
    }
}
