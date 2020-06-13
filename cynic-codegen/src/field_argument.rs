use proc_macro2::TokenStream;

use super::field_type::FieldType;
use super::type_path::TypePath;
use crate::{schema, Ident, TypeIndex};

#[derive(Debug, Clone)]
pub struct FieldArgument {
    pub(crate) name: Ident,
    pub(crate) argument_type: FieldType,
    pub(crate) gql_name: String,
    pub(crate) gql_type: String,
}

impl FieldArgument {
    pub fn from_input_value(value: &schema::InputValue, type_index: &TypeIndex) -> Self {
        use crate::schema::TypeExt;

        FieldArgument {
            name: Ident::for_field(&value.name),
            argument_type: FieldType::from_schema_type(&value.value_type, type_index),
            gql_type: value.value_type.to_graphql_string(),
            gql_name: value.name.clone(),
        }
    }

    pub fn is_required(&self) -> bool {
        !self.argument_type.is_nullable()
    }

    pub fn generic_parameter(&self) -> Option<GenericParameter> {
        if let Some(path) = self.argument_type.inner_enum_path() {
            Some(GenericParameter {
                name: Ident::for_type(format!("{}T", self.name)),
                constraint: GenericConstraint::Enum(path),
            })
        } else if let Some(path) = self.argument_type.inner_input_object_path() {
            Some(GenericParameter {
                name: Ident::for_type(format!("{}T", self.name)),
                constraint: GenericConstraint::InputObject(path),
            })
        } else {
            None
        }
    }
}

/// A GenericParameter, which struct fields may or may not require.
///
/// We use these for input struct or enum arguments. We're expecting users
/// to define these types but we need to take them as arguements so we make
/// our argument structs generic, and apply constraints to make sure the
/// correct types are passed in.
pub struct GenericParameter {
    pub name: Ident,
    pub constraint: GenericConstraint,
}

impl GenericParameter {
    pub fn to_tokens(&self, path_to_markers: TypePath) -> TokenStream {
        use quote::quote;

        let name = &self.name;
        let constraint = self.constraint.to_tokens(path_to_markers);

        quote! {
            #name: #constraint
        }
    }
}

/// Our generic parameters need constraints - this enum specifies what they
/// should be.
pub enum GenericConstraint {
    /// An enum type constraint: `where T: Enum<SomeEnumMarkerStruct>
    Enum(Ident),
    /// An input object constraint: `where T: InputObject<SomeInputObjectMarkerStruct>
    InputObject(Ident),
}

impl GenericConstraint {
    fn to_tokens(&self, path_to_markers: TypePath) -> TokenStream {
        use quote::quote;

        match self {
            GenericConstraint::Enum(ident) => {
                let type_path = TypePath::concat(&[path_to_markers, ident.clone().into()]);

                quote! { ::cynic::Enum<#type_path> + ::cynic::SerializableArgument }
            }
            GenericConstraint::InputObject(ident) => {
                let type_path = TypePath::concat(&[path_to_markers, ident.clone().into()]);

                quote! { ::cynic::InputObject<#type_path> + ::cynic::SerializableArgument }
            }
        }
    }
}
