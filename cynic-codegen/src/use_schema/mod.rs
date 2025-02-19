mod argument;
mod directive;
mod fields;
mod input_object;
mod interface;
mod named_type;
mod object;
mod params;
mod schema_roots;
mod subtype_markers;

pub use params::UseSchemaParams;

use {
    proc_macro2::TokenStream,
    quote::{quote, ToTokens},
};

use crate::{
    error::Errors,
    schema::{
        types::{DirectiveLocation, Type},
        Schema, SchemaInput, Validated,
    },
};

use self::{
    directive::FieldDirectiveOutput, input_object::InputObjectOutput, interface::InterfaceOutput,
    named_type::NamedType, object::ObjectOutput, subtype_markers::SubtypeMarkers,
};

pub fn use_schema(input: UseSchemaParams) -> Result<TokenStream, Errors> {
    let input = SchemaInput::from_schema_path(input.schema_filename)
        .map_err(|e| e.into_syn_error(proc_macro2::Span::call_site()))?;

    let schema = Schema::new(input).validate()?;
    use_schema_impl(&schema)
}

pub(crate) fn use_schema_impl(schema: &Schema<'_, Validated>) -> Result<TokenStream, Errors> {
    use quote::TokenStreamExt;

    let mut output = TokenStream::new();
    let mut field_module = TokenStream::new();

    let root_types = schema.root_types()?;
    output.append_all(quote! {
        #root_types
    });

    let mut subtype_markers = Vec::new();
    let mut named_types = Vec::new();

    for definition in schema.iter() {
        named_types.extend(NamedType::from_def(&definition));

        match definition {
            Type::Scalar(def) if !def.builtin => {
                let name = proc_macro2::Literal::string(def.name.as_ref());
                let ident = def.marker_ident().to_rust_ident();
                output.append_all(quote! {
                    pub struct #ident {}
                    impl cynic::schema::NamedType for #ident {
                        const NAME: &'static ::core::primitive::str = #name;
                    }
                });
            }
            Type::Scalar(_) => {}
            Type::Object(def) => {
                subtype_markers.extend(SubtypeMarkers::from_object(&def));

                let object = ObjectOutput::new(def);
                object.to_tokens(&mut output);
                object.append_fields(&mut field_module);
            }
            Type::Interface(def) => {
                subtype_markers.push(SubtypeMarkers::from_interface(&def));

                let iface = InterfaceOutput::new(def);
                iface.to_tokens(&mut output);
                iface.append_fields(&mut field_module);
            }
            Type::Union(def) => {
                subtype_markers.extend(SubtypeMarkers::from_union(&def));

                let ident = def.marker_ident().to_rust_ident();
                output.append_all(quote! {
                    pub struct #ident {}
                });
            }
            Type::Enum(def) => {
                let ident = def.marker_ident().to_rust_ident();
                output.append_all(quote! {
                    pub struct #ident {}
                });
            }
            Type::InputObject(def) => {
                let object = InputObjectOutput::new(def);
                object.to_tokens(&mut output);
                object.append_fields(&mut field_module);
            }
        }
    }

    for directive in schema.directives() {
        if !directive.locations.contains(&DirectiveLocation::Field) {
            // We only support field directives for now
            continue;
        }
        FieldDirectiveOutput {
            directive: &directive,
        }
        .to_tokens(&mut output);
    }

    output.append_all(quote! {
        #(#subtype_markers)*
        #(#named_types)*

        #[allow(non_snake_case, non_camel_case_types)]
        pub mod __fields {
            #field_module
        }

        pub type Boolean = bool;
        pub type String = std::string::String;
        pub type Float = f64;
        pub type Int = i32;
        pub type ID = cynic::Id;

        pub mod variable {
            use cynic::{
                variables::VariableType,
                serde::{Serialize, Serializer, ser::SerializeSeq}
            };

            /// Used to determine the type of a given variable that
            /// appears in a variable struct.
            pub trait Variable {
                const TYPE: VariableType;

                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer;
            }

            pub struct VariableSerialize<T>(pub T);

            impl<T> Serialize for VariableSerialize<T> where T: Variable {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer
                {
                    Variable::serialize(&self.0, serializer)
                }
            }

            impl<T> Variable for &T
            where
                T: ?::core::marker::Sized + Variable,
            {
                const TYPE: VariableType = T::TYPE;

                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer
                {
                    <T as Variable>::serialize(self, serializer)
                }
            }

            impl<T> Variable for Option<T>
            where
                T: Variable
            {
                const TYPE: VariableType = VariableType::Nullable(&T::TYPE);

                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer
                {
                    match self {
                        Some(inner) => {
                            serializer.serialize_some(&VariableSerialize(inner))
                        },
                        None => {
                            serializer.serialize_none()
                        }
                    }
                }
            }

            impl<T> Variable for [T]
            where
                T: Variable,
            {
                const TYPE: VariableType = VariableType::List(&T::TYPE);

                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer
                {
                    let mut seq = serializer.serialize_seq(Some(self.len()))?;
                    for element in self {
                        seq.serialize_element(&VariableSerialize(element))?;
                    }
                    seq.end()
                }
            }

            impl<T> Variable for Vec<T>
            where
                T: Variable,
            {
                const TYPE: VariableType = VariableType::List(&T::TYPE);

                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer
                {
                    let mut seq = serializer.serialize_seq(Some(self.len()))?;
                    for element in self {
                        seq.serialize_element(&VariableSerialize(element))?;
                    }
                    seq.end()
                }
            }

            impl<T> Variable for Box<T>
            where
                T: Variable,
            {
                const TYPE: VariableType = T::TYPE;

                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer
                {
                    <T as Variable>::serialize(self.as_ref(), serializer)
                }
            }

            impl<T> Variable for std::rc::Rc<T>
            where
                T: Variable,
            {
                const TYPE: VariableType = T::TYPE;

                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer
                {
                    <T as Variable>::serialize(self.as_ref(), serializer)
                }
            }

            impl<T> Variable for std::sync::Arc<T>
            where
                T: Variable,
            {
                const TYPE: VariableType = T::TYPE;

                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer
                {
                    <T as Variable>::serialize(self.as_ref(), serializer)
                }
            }

            impl<T> Variable for std::borrow::Cow<'_, T>
            where
                T: ?::core::marker::Sized + Variable + ToOwned,
            {
                const TYPE: VariableType = T::TYPE;

                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer
                {
                    <T as Variable>::serialize(self.as_ref(), serializer)
                }
            }

            impl Variable for bool {
                const TYPE: VariableType = VariableType::Named("Boolean");

                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer
                {
                    <Self as cynic::serde::Serialize>::serialize(self, serializer)
                }
            }

            impl Variable for str {
                const TYPE: VariableType = VariableType::Named("String");

                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer
                {
                    <Self as cynic::serde::Serialize>::serialize(self, serializer)
                }
            }

            impl Variable for String {
                const TYPE: VariableType = <str as Variable>::TYPE;

                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer
                {
                    <Self as cynic::serde::Serialize>::serialize(self, serializer)
                }
            }

            impl Variable for f64 {
                const TYPE: VariableType = VariableType::Named("Float");

                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer
                {
                    <Self as cynic::serde::Serialize>::serialize(self, serializer)
                }
            }

            impl Variable for i32 {
                const TYPE: VariableType = VariableType::Named("Int");

                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer
                {
                    <Self as cynic::serde::Serialize>::serialize(self, serializer)
                }
            }

            impl Variable for cynic::Id {
                const TYPE: VariableType = VariableType::Named("ID");

                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer
                {
                    <Self as cynic::serde::Serialize>::serialize(self, serializer)
                }
            }
        }
    });

    Ok(output)
}
