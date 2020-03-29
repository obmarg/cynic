// TODO: Ok, so this is a bit weird.
// The query is heirarchical, but the schema is not.
// So we need to take the schema and build up a heirarchy, then flatten that, indexed by position.
// WOW Argh.

use graphql_parser::query::Type;
use graphql_parser::schema::{Definition, Document, TypeDefinition};
use std::collections::HashMap;

use crate::Error;

pub struct TypeIndex<'a> {
    types: HashMap<&'a str, GraphqlType<'a>>,
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
                    Some((scalar.name, GraphqlType::Scalar))
                }
                Definition::TypeDefinition(TypeDefinition::Object(obj)) => {
                    let fields = obj
                        .fields
                        .iter()
                        .map(|field| (field.name, &field.field_type))
                        .collect();

                    Some((obj.name, GraphqlType::Object(fields)))
                }
                Definition::TypeDefinition(TypeDefinition::Enum(en)) => {
                    Some((en.name, GraphqlType::Enum))
                }
                _ => None,
            })
            .flatten()
            .collect::<HashMap<_, _>>();

        types.insert("String", GraphqlType::Scalar);
        types.insert("Int", GraphqlType::Scalar);
        types.insert("Boolean", GraphqlType::Scalar);
        types.insert("ID", GraphqlType::Scalar);

        TypeIndex {
            types,
            root: "Query".into(),
        }
    }

    pub fn type_for_path<'b>(&self, path: &[&'b str]) -> Result<&'a Type<'a, &'a str>, Error> {
        // TODO: tidy up unwraps etc.
        let root = self.types.get(self.root.as_str()).unwrap();
        if let GraphqlType::Object(root_fields) = root {
            self.find_type_recursive(root_fields, self.root.as_str(), path)
        } else {
            panic!("TODO: make this an error");
        }
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
                let inner_name = inner_name(fields.get(first).ok_or(Error::UnknownField(
                    first.to_string(),
                    current_type_name.to_string(),
                ))?);

                let inner_type = self.types.get(inner_name).unwrap();

                if let GraphqlType::Object(fields) = inner_type {
                    self.find_type_recursive(fields, &inner_name, rest)
                } else {
                    panic!("TODO: make this an error");
                }
            }
        }
    }
}

#[derive(Debug, PartialEq)]
enum GraphqlType<'a> {
    Enum,
    Object(HashMap<&'a str, &'a Type<'a, &'a str>>),
    Scalar,
}

pub fn inner_name<'a>(ty: &Type<'a, &'a str>) -> &'a str {
    match ty {
        Type::NamedType(s) => s,
        Type::ListType(inner) => inner_name(inner),
        Type::NonNullType(inner) => inner_name(inner),
    }
}
