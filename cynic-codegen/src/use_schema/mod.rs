//mod model;
mod named_type;
mod object;
mod params;
mod schema_roots;
mod subtype_markers;

pub use params::UseSchemaParams;

use proc_macro2::TokenStream;
use quote::{format_ident, ToTokens};
use syn::parse_quote;

use crate::{
    error::Errors,
    field_type::FieldType,
    idents::{to_snake_case, Ident},
    schema::{self, types::Type, Schema, TypeIndex},
};

use self::{named_type::NamedType, object::ObjectOutput, subtype_markers::SubtypeMarkers};

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

                ObjectOutput::new(def).to_tokens(&mut output);
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

        pub type Boolean = bool;
        pub type String = std::string::String;
        pub type Float = f64;
        pub type Int = i32;
        pub type Id = cynic::Id;
    });

    Ok(output)
}
