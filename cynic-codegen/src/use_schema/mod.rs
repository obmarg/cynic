mod input_object;
mod named_type;
mod object;
mod params;
mod schema_roots;
mod subtype_markers;

pub use params::UseSchemaParams;

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::parse_quote;

use crate::{
    error::Errors,
    idents::{to_snake_case, Ident},
    schema::{
        self,
        types::{InputType, Type, TypeRef},
        Schema, TypeIndex,
    },
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
        pub type Id = ::cynic::Id;

        pub mod variable {
            use ::cynic::core::VariableType;

            /// Used to determine the type of a given variable that
            /// appears in an argument struct.
            pub trait Variable {
                const TYPE: VariableType;
            }

            impl<T> Variable for &T where T: Variable {
                const TYPE: VariableType = T::TYPE;
            }

            impl<T> Variable for Option<T>
            where
                T: Variable
            {
                const TYPE: VariableType = VariableType::Nullable(&T::TYPE);
            }

            impl<T> Variable for Vec<T>
            where
                T: Variable,
            {
                const TYPE: VariableType = VariableType::List(&T::TYPE);
            }

            impl<T> Variable for Box<T>
            where
                T: Variable,
            {
                const TYPE: VariableType = T::TYPE;
            }

            impl<T> Variable for std::rc::Rc<T>
            where
                T: Variable,
            {
                const TYPE: VariableType = T::TYPE;
            }

            impl<T> Variable for std::sync::Arc<T>
            where
                T: Variable,
            {
                const TYPE: VariableType = T::TYPE;
            }

            impl Variable for bool {
                const TYPE: VariableType = VariableType::Named("Boolean");
            }

            impl Variable for String {
                const TYPE: VariableType = VariableType::Named("String");
            }

            impl Variable for f64 {
                const TYPE: VariableType = VariableType::Named("Float");
            }

            impl Variable for i32 {
                const TYPE: VariableType = VariableType::Named("Int");
            }

            impl Variable for ::cynic::Id {
                const TYPE: VariableType = VariableType::Named("ID");
            }
        }
    });

    Ok(output)
}

impl TypeRef<'_, InputType<'_>> {
    fn argument_kind(&self) -> proc_macro2::TokenStream {
        match self.inner_type() {
            InputType::Scalar(_) => {
                quote! {
                    ::cynic::schema::ScalarArgument
                }
            }
            InputType::Enum(_) => {
                quote! {
                    ::cynic::schema::EnumArgument
                }
            }
            InputType::InputObject(_) => quote! {
                ::cynic::schema::InputObjectArgument
            },
        }
    }
}
