use crate::{Errors, FieldType, Ident, TypePath};
use proc_macro2::{Span, TokenStream};

/// A parmeter to a query_dsl function that represents a GraphQL parameter.
pub struct ArgumentParameter {
    name: Ident,
    param_type: ArgumentParameterType,
}

impl ArgumentParameter {
    pub fn new(name: Ident, argument_type: FieldType) -> ArgumentParameter {
        ArgumentParameter {
            name,
            param_type: ArgumentParameterType::from_type(argument_type),
        }
    }

    pub fn to_tokens(&self, path_to_markers: TypePath) -> Result<TokenStream, Errors> {
        use quote::quote;

        let name = &self.name;
        let type_tokens = self.param_type.to_tokens(path_to_markers)?;

        Ok(quote! { #name: #type_tokens })
    }
}

pub struct ArgumentParameterType {
    argument_type: FieldType,
}

impl ArgumentParameterType {
    pub fn from_type(argument_type: FieldType) -> ArgumentParameterType {
        ArgumentParameterType { argument_type }
    }

    pub fn to_tokens(&self, path_to_markers: TypePath) -> Result<TokenStream, Errors> {
        use quote::quote;

        // TODO: Figure out if we can get rid of this generic_inner_type parameter...
        let type_lock = self.argument_type.to_tokens(None, path_to_markers.clone());
        if self.argument_type.contains_enum() {
            Ok(quote! { impl ::cynic::EnumArgument<#type_lock> })
        } else if self.argument_type.contains_scalar() {
            let scalar_lock = self.argument_type.as_type_lock(path_to_markers);
            Ok(quote! { impl ::cynic::ScalarArgument<#scalar_lock, #type_lock> })
        } else if self.argument_type.contains_input_object() {
            Ok(quote! { impl ::cynic::InputObjectArgument<#type_lock> })
        } else {
            Err(syn::Error::new(
                Span::call_site(),
                "Arguments must be scalars, enums or input objects",
            )
            .into())
        }
    }
}
