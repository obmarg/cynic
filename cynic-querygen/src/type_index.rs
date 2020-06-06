use graphql_parser::query::Type;
use graphql_parser::schema::{Definition, Document, EnumType, ScalarType, TypeDefinition};
use std::collections::HashMap;

use crate::{type_ext::TypeExt, Error};

pub struct TypeIndex<'a> {
    types: HashMap<&'a str, FieldType<'a>>,
    root: String,
}

type FieldMap<'a> = HashMap<&'a str, &'a Type<'a, &'a str>>;

impl<'a> TypeIndex<'a> {
    pub fn from_schema<'b>(schema: &'b Document<'b, &'b str>) -> TypeIndex<'b> {
        let mut types = schema
            .definitions
            .iter()
            .map(|definition| match definition {
                Definition::TypeDefinition(TypeDefinition::Scalar(scalar)) => {
                    Some((scalar.name, FieldType::Scalar(ScalarKind::Custom)))
                }
                Definition::TypeDefinition(TypeDefinition::Object(obj)) => {
                    let fields = obj
                        .fields
                        .iter()
                        .map(|field| (field.name, &field.field_type))
                        .collect();

                    Some((obj.name, FieldType::Object(fields)))
                }
                Definition::TypeDefinition(TypeDefinition::Enum(en)) => {
                    Some((en.name, FieldType::Enum(en)))
                }
                _ => None,
            })
            .flatten()
            .collect::<HashMap<_, _>>();

        let mut rv = TypeIndex::default();
        rv.types.extend(types);

        rv
    }

    pub fn type_for_path<'b>(&self, path: &[&'b str]) -> Result<&'a Type<'a, &'a str>, Error> {
        // TODO: tidy up unwraps etc.
        let root = self.types.get(self.root.as_str()).unwrap();
        if let FieldType::Object(root_fields) = root {
            self.find_type_recursive(root_fields, self.root.as_str(), path)
        } else {
            panic!("TODO: make this an error");
        }
    }

    pub fn lookup_type(&self, name: &str) -> Option<&FieldType<'a>> {
        self.types.get(name)
    }

    fn find_type_recursive<'b>(
        &self,
        fields: &FieldMap<'a>,
        current_type_name: &'b str,
        path: &[&'b str],
    ) -> Result<&'a Type<'a, &'a str>, Error> {
        // TODO: tidy up unwraps etc.
        match path {
            [] => panic!("This shouldn't happen"),
            [first] => fields.get(first).map(|f| *f).ok_or(Error::UnknownField(
                first.to_string(),
                current_type_name.to_string(),
            )),
            [first, rest @ ..] => {
                let inner_name = fields
                    .get(first)
                    .ok_or(Error::UnknownField(
                        first.to_string(),
                        current_type_name.to_string(),
                    ))?
                    .inner_name();

                let inner_type = self.types.get(inner_name).unwrap();

                if let FieldType::Object(fields) = inner_type {
                    self.find_type_recursive(fields, &inner_name, rest)
                } else {
                    panic!("TODO: make this an error");
                }
            }
        }
    }
}

impl<'a> Default for TypeIndex<'a> {
    fn default() -> TypeIndex<'a> {
        let mut types = HashMap::new();

        types.insert("String", FieldType::Scalar(ScalarKind::BuiltIn));
        types.insert("Int", FieldType::Scalar(ScalarKind::BuiltIn));
        types.insert("Boolean", FieldType::Scalar(ScalarKind::BuiltIn));
        types.insert("ID", FieldType::Scalar(ScalarKind::BuiltIn));

        TypeIndex {
            types,
            root: "Query".into(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ScalarKind {
    BuiltIn,
    Custom,
}

#[derive(Debug, PartialEq)]
pub enum FieldType<'a> {
    Enum(&'a EnumType<'a, &'a str>),
    Object(HashMap<&'a str, &'a Type<'a, &'a str>>),
    Scalar(ScalarKind),
}
