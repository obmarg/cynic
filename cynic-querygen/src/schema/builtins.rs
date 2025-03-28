use cynic_parser::{
    common::TypeWrappers,
    type_system::{
        ids::FieldDefinitionId,
        storage::{
            DirectiveDefinitionRecord, FieldDefinitionRecord, InputValueDefinitionRecord,
            ScalarDefinitionRecord, TypeRecord,
        },
        writer, DirectiveLocation,
    },
    TypeSystemDocument,
};

pub(crate) fn add_builtins(schema: TypeSystemDocument) -> (TypeSystemDocument, FieldDefinitionId) {
    let mut writer = writer::TypeSystemAstWriter::update(schema);

    // Add the builtins
    for name in ["String", "Int", "Float", "Boolean", "ID"] {
        let name = writer.intern_string(name);
        writer.scalar_definition(ScalarDefinitionRecord {
            name,
            description: None,
            directives: Default::default(),
            span: Default::default(),
            name_span: Default::default(),
        });
    }

    let typename_id = {
        let ty = {
            let name = writer.intern_string("String");
            writer.type_reference(TypeRecord {
                name,
                name_start: Default::default(),
                wrappers: TypeWrappers::none().wrap_non_null(),
                span: Default::default(),
            })
        };

        let name = writer.intern_string("__typename");
        writer.field_definition(FieldDefinitionRecord {
            name,
            name_span: Default::default(),
            ty,
            arguments: Default::default(),
            description: Default::default(),
            directives: Default::default(),
            span: Default::default(),
        })
    };

    let bool = writer.intern_string("Boolean");
    for name in ["skip", "include"] {
        let arguments = {
            let ty = writer.type_reference(TypeRecord {
                name: bool,
                name_start: Default::default(),
                wrappers: TypeWrappers::none().wrap_non_null(),
                span: Default::default(),
            });
            let name = writer.intern_string("if");
            writer.input_value_definition(InputValueDefinitionRecord {
                name,
                name_span: Default::default(),
                ty,
                description: None,
                default_value: None,
                default_value_span: Default::default(),
                directives: Default::default(),
                span: Default::default(),
            });
            writer.input_value_definition_range(Some(1))
        };

        let name = writer.intern_string(name);
        writer.directive_definition(DirectiveDefinitionRecord {
            name,
            name_span: Default::default(),
            description: None,
            arguments,
            is_repeatable: false,
            locations: vec![
                DirectiveLocation::Field,
                DirectiveLocation::FragmentSpread,
                DirectiveLocation::InlineFragment,
            ],
            span: Default::default(),
        });
    }

    (writer.finish(), typename_id)
}
