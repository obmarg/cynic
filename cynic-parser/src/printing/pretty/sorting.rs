use crate::type_system::Definition;

#[derive(PartialOrd, Ord, PartialEq, Eq)]
pub(super) enum DefinitionSortKey<'a> {
    Directive(&'a str),
    Type(&'a str, bool),
    Schema(bool),
}

pub(super) fn sort_key_for<'a>(definition: &Definition<'a>) -> DefinitionSortKey<'a> {
    match definition {
        Definition::Schema(_) => DefinitionSortKey::Schema(false),
        Definition::SchemaExtension(_) => DefinitionSortKey::Schema(true),
        Definition::Type(def) => DefinitionSortKey::Type(def.name(), false),
        Definition::TypeExtension(def) => DefinitionSortKey::Type(def.name(), true),
        Definition::Directive(def) => DefinitionSortKey::Directive(def.name()),
    }
}
