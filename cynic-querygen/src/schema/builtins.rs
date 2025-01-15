use cynic_parser::{
    common::TypeWrappers,
    type_system::{
        ids::FieldDefinitionId,
        storage::{FieldDefinitionRecord, ScalarDefinitionRecord, TypeRecord},
        writer,
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

    (writer.finish(), typename_id)
}
