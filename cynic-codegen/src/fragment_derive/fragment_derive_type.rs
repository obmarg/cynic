use crate::schema::{
    markers::{FieldMarkerModule, MarkerIdent},
    types::{Field, Kind, Type},
    FieldName, SchemaError,
};

pub struct FragmentDeriveType<'a> {
    pub fields: Vec<Field<'a>>,
    pub name: &'a str,
    pub marker_ident: MarkerIdent<'a>,
    pub field_module: FieldMarkerModule<'a>,
}

impl<'a> FragmentDeriveType<'a> {
    pub fn field<N>(&self, name: &N) -> Option<&Field<'a>>
    where
        for<'b> FieldName<'b>: PartialEq<N>,
    {
        self.fields.iter().find(|field| field.name == *name)
    }
}

impl<'a> TryFrom<Type<'a>> for FragmentDeriveType<'a> {
    type Error = SchemaError;

    fn try_from(value: Type<'a>) -> Result<Self, Self::Error> {
        match value {
            Type::Interface(iface) => Ok(FragmentDeriveType {
                marker_ident: iface.marker_ident(),
                field_module: iface.field_module(),
                name: iface.name,
                fields: iface.fields,
            }),
            Type::Object(obj) => Ok(FragmentDeriveType {
                marker_ident: obj.marker_ident(),
                field_module: obj.field_module(),
                name: obj.name,
                fields: obj.fields,
            }),
            other => Err(SchemaError::unexpected_kind(other, Kind::ObjectOrInterface)),
        }
    }
}
