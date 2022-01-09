//mod model;
mod named_type;
mod params;
mod schema_roots;
mod subtype_markers;

pub use params::UseSchemaParams;

use proc_macro2::TokenStream;
use quote::format_ident;
use syn::parse_quote;

use crate::{
    error::Errors,
    field_type::FieldType,
    idents::{to_snake_case, Ident},
    schema::{self, types::Type, Schema, TypeIndex},
};

use self::{named_type::NamedType, subtype_markers::SubtypeMarkers};

pub fn use_schema(input: UseSchemaParams) -> Result<TokenStream, Errors> {
    use quote::{quote, TokenStreamExt};

    let document = crate::schema::load_schema(input.schema_filename)
        .map_err(|e| e.into_syn_error(proc_macro2::Span::call_site()))?;
    let schema = Schema::new(&document).validate()?;

    let mut output = TokenStream::new();

    let root_types = schema_roots::RootTypes::from_definitions(&document.definitions);
    output.append_all(quote! {
        #root_types
    });

    let mut subtype_markers = Vec::new();
    let mut named_types = Vec::new();

    for definition in schema.iter() {
        named_types.extend(NamedType::from_def(&definition));

        match definition {
            Type::Scalar(def) if !def.builtin => {
                let ident = Ident::for_type(&def.name);
                output.append_all(quote! {
                    pub struct #ident {}
                });
            }
            Type::Scalar(_) => {}
            Type::Object(def) => {
                subtype_markers.extend(SubtypeMarkers::from_object(&def));

                let object_marker = proc_macro2::Ident::from(def.marker_ident());
                output.append_all(quote! {
                    pub struct #object_marker;
                });

                let field_module = proc_macro2::Ident::from(def.field_module());
                let mut field_module_contents = Vec::new();
                for field in def.fields {
                    let field_marker_struct = proc_macro2::Ident::from(field.marker_ident());
                    let field_name_literal = proc_macro2::Literal::string(field.name.as_str());

                    let field_type_marker = field
                        .field_type
                        .marker_type()
                        .to_path(&parse_quote! { super });

                    field_module_contents.push(quote! {
                        pub struct #field_marker_struct;

                        impl ::cynic::schema::Field for #field_marker_struct {
                            type SchemaType = #field_type_marker;

                            fn name() -> &'static str {
                                #field_name_literal
                            }
                        }

                        impl ::cynic::schema::HasField<#field_marker_struct, #field_type_marker> for super::#object_marker {}

                        // TODO: implement HasField for all the valid conversions...
                        // assuming that's even possible - implementing the deserialize might be tricky for
                        // some of them?  Need to check what's even allowed here...
                    })

                    // TODO: Handle arguments
                }

                output.append_all(quote! {
                    pub mod #field_module {
                        #(#field_module_contents)*
                    }
                });
            }
            Type::Interface(def) => {
                subtype_markers.push(SubtypeMarkers::from_interface(&def));

                let ident = Ident::for_type(&def.name);
                output.append_all(quote! {
                    pub struct #ident {}
                });

                // TODO: the rest of this.  Presumably we need fields & HasSubtype
            }
            Type::Union(def) => {
                subtype_markers.extend(SubtypeMarkers::from_union(&def));

                let ident = Ident::for_type(&def.name);
                output.append_all(quote! {
                    pub struct #ident {}
                });

                // TODO: the rest of this.  Presumably we need just HasSubtype
            }
            Type::Enum(def) => {
                let ident = Ident::for_type(&def.name);
                output.append_all(quote! {
                    pub struct #ident {}
                });
            }
            Type::InputObject(def) => {
                let ident = Ident::for_type(&def.name);
                output.append_all(quote! {
                    pub struct #ident {}
                });

                // TODO: Handle fields
            }
        }
    }

    output.append_all(quote! {
        #(#subtype_markers)*
        #(#named_types)*

        type Boolean = bool;
        type String = std::string::String;
        type Float = f64;
        type Int = i32;
        type Id = cynic::Id;
    });

    Ok(output)
}

fn type_def_from_definition(definition: schema::Definition) -> Option<schema::TypeDefinition> {
    match definition {
        graphql_parser::schema::Definition::TypeDefinition(inner) => Some(inner),
        _ => None,
    }
}
