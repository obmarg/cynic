use core::fmt;

use graphql_parser::Pos;
use inflector::Inflector;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};

use super::indented;
use crate::{query_parsing::TypedValue, schema::OutputFieldType, Error};

#[derive(Debug, PartialEq)]
pub struct QueryFragment<'query, 'schema> {
    pub fields: Vec<OutputField<'query, 'schema>>,
    pub target_type: String,
    pub argument_struct_name: Option<String>,

    pub name: String,
}

// impl std::fmt::Display for QueryFragment<'_, '_> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         writeln!(f, "#[derive(cynic::QueryFragment, Debug)]")?;

//         if self.target_type != self.name || self.argument_struct_name.is_some() {
//             write!(f, "#[cynic(")?;
//             if self.target_type != self.name {
//                 write!(f, "graphql_type = \"{}\"", self.target_type)?;
//             }

//             if let Some(name) = &self.argument_struct_name {
//                 if self.target_type != self.name {
//                     write!(f, ", ")?;
//                 }
//                 write!(f, "argument_struct = \"{}\"", name)?;
//             }
//             writeln!(f, ")]",)?;
//         }

//         writeln!(f, "pub struct {} {{", self.name)?;
//         for field in &self.fields {
//             write!(indented(f, 4), "{}", field)?;
//         }

//         writeln!(f, "}}")
//     }
// }

impl QueryFragment<'_, '_> {
    pub fn fmt<F: fmt::Write + ?Sized>(&self, f: &mut F) -> Result<(), Error> {
        writeln!(f, "#[derive(cynic::QueryFragment, Debug)]").unwrap();

        if self.target_type != self.name || self.argument_struct_name.is_some() {
            write!(f, "#[cynic(").unwrap();
            if self.target_type != self.name {
                write!(f, "graphql_type = \"{}\"", self.target_type).unwrap();
            }

            if let Some(name) = &self.argument_struct_name {
                if self.target_type != self.name {
                    write!(f, ", ").unwrap();
                }
                write!(f, "argument_struct = \"{}\"", name).unwrap();
            }
            writeln!(f, ")]",).unwrap();
        }

        writeln!(f, "pub struct {} {{", self.name).unwrap();
        for field in &self.fields {
            field.fmt(&mut indented(f, 4))?;
        }

        writeln!(f, "}}").unwrap();

        Ok(())
    }
}

impl ToTokens for QueryFragment<'_, '_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let description = if self.target_type != self.name || self.argument_struct_name.is_some() {
            let target_type = if self.target_type != self.name {
                let target_type = &self.target_type;
                Some(quote! {graphql_type = #target_type})
            } else {
                None
            };

            let argument_struct = if let Some(name) = &self.argument_struct_name {
                let comma = if target_type.is_some() {
                    Some(quote! {,})
                } else {
                    None
                };
                Some(quote! {#comma argument_struct = #name})
            } else {
                None
            };

            Some(quote! {
                #[cynic(#target_type #argument_struct)]
            })
        } else {
            None
        };

        let name = Ident::new(&self.name, Span::call_site());
        let fields = &self.fields;

        tokens.extend(quote! {
            #[derive(cynic::QueryFragment, Debug)]
            #description
            pub struct #name {
                #(#fields)*
            }
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct OutputField<'query, 'schema> {
    pub name: &'schema str,
    pub rename: Option<&'schema str>,
    pub field_type: RustOutputFieldType,

    pub arguments: Vec<FieldArgument<'query, 'schema>>,
}

// impl std::fmt::Display for OutputField<'_, '_> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         if !self.arguments.is_empty() {
//             let arguments_string = self
//                 .arguments
//                 .iter()
//                 .map(|arg| format!("{} = {}", arg.name.0.to_snake_case(), arg.to_literal()?))
//                 .collect::<Vec<_>>()
//                 .join(", ");

//             writeln!(f, "#[arguments({})]", arguments_string)?;
//         }

//         if let Some(rename) = self.rename {
//             writeln!(f, "#[cynic(rename = \"{}\")]", rename)?;
//         }

//         writeln!(
//             f,
//             "pub {}: {},",
//             self.name.to_snake_case(),
//             self.field_type.type_spec()
//         )
//     }
// }

impl OutputField<'_, '_> {
    pub fn fmt<F: fmt::Write + ?Sized>(&self, f: &mut F) -> Result<(), Error> {
        if !self.arguments.is_empty() {
            let arguments_string = self
                .arguments
                .iter()
                .map(|arg| {
                    Ok(format!(
                        "{} = {}",
                        arg.name.0.to_snake_case(),
                        arg.to_literal()?
                    ))
                })
                .collect::<Result<Vec<_>, Error>>()?
                .join(", ");

            writeln!(f, "#[arguments({})]", arguments_string).unwrap();
        }

        if let Some(rename) = self.rename {
            writeln!(f, "#[cynic(rename = \"{}\")]", rename).unwrap();
        }

        writeln!(
            f,
            "pub {}: {},",
            self.name.to_snake_case(),
            self.field_type.type_spec()
        )
        .unwrap();

        Ok(())
    }
}

impl ToTokens for OutputField<'_, '_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let arguments = self.arguments.iter().map(|arg| {
            let name = Ident::new(&arg.name.0.to_snake_case(), Span::call_site());
            let lit = Ident::new(&arg.to_literal().unwrap(), Span::call_site());
            quote! { #name = #lit, }
        });

        let arguments = if arguments.clone().count() > 0 {
            Some(quote! {#[arguments(#(#arguments)*)]})
        } else {
            None
        };

        let rename = self.rename.map(|r| quote! {#[cynic(rename = #r)]});

        let name = Ident::new(&self.name.to_snake_case(), Span::call_site());
        let typ: TokenStream = self.field_type.type_spec().parse().unwrap();

        tokens.extend(quote! {
            #arguments

            #rename

            pub #name: #typ,
        })
    }
}

/// An OutputFieldType that has been given a rust-land name.  Allows for
/// the fact that there may be several rust structs that refer to the same schema
/// type.
#[derive(Debug, PartialEq)]
pub enum RustOutputFieldType {
    NamedType(String),
    ListType(Box<RustOutputFieldType>),
    NonNullType(Box<RustOutputFieldType>),
}

impl RustOutputFieldType {
    pub fn from_schema_type(
        schema_type: &OutputFieldType<'_>,
        inner_name: String,
    ) -> RustOutputFieldType {
        match schema_type {
            OutputFieldType::NonNullType(inner) => RustOutputFieldType::NonNullType(Box::new(
                RustOutputFieldType::from_schema_type(inner, inner_name),
            )),
            OutputFieldType::ListType(inner) => RustOutputFieldType::ListType(Box::new(
                RustOutputFieldType::from_schema_type(inner, inner_name),
            )),
            OutputFieldType::NamedType(_) => RustOutputFieldType::NamedType(inner_name),
        }
    }

    pub fn type_spec(&self) -> String {
        self.output_type_spec_imp(true)
    }

    fn output_type_spec_imp(&self, nullable: bool) -> String {
        if let RustOutputFieldType::NonNullType(inner) = self {
            return inner.output_type_spec_imp(false);
        }

        if nullable {
            return format!("Option<{}>", self.output_type_spec_imp(false));
        }

        match self {
            RustOutputFieldType::ListType(inner) => {
                format!("Vec<{}>", inner.output_type_spec_imp(true))
            }

            RustOutputFieldType::NonNullType(_) => panic!("NonNullType somehow got past an if let"),

            RustOutputFieldType::NamedType(s) => {
                match s.as_ref() {
                    "Int" => return "i32".into(),
                    "Float" => return "f64".into(),
                    "Boolean" => return "bool".into(),
                    "ID" => return "cynic::Id".into(),
                    _ => {}
                }

                s.clone()
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct FieldArgument<'query, 'schema> {
    pub name: (&'schema str, Pos),
    value: TypedValue<'query, 'schema>,
}

impl<'query, 'schema> FieldArgument<'query, 'schema> {
    pub fn new(name: (&'schema str, Pos), value: TypedValue<'query, 'schema>) -> Self {
        FieldArgument { name, value }
    }

    pub fn to_literal(&self) -> Result<String, Error> {
        use crate::query_parsing::LiteralContext;

        self.value.to_literal(LiteralContext::Argument)
    }
}
