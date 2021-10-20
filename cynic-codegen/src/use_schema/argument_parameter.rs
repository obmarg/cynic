use crate::{Errors, FieldType, Ident};
use proc_macro2::TokenStream;

/// A parmeter to a schema function that represents a GraphQL parameter.
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

    pub fn to_tokens(&self, path_to_markers: syn::Path) -> Result<TokenStream, Errors> {
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

    pub fn to_tokens(&self, path_to_markers: syn::Path) -> Result<TokenStream, Errors> {
        use quote::quote;

        let type_lock = self.argument_type.as_type_lock(&path_to_markers);
        let wrapper_path = self.argument_type.wrapper_path()?;

        Ok(quote! { impl ::cynic::InputType<#type_lock, #wrapper_path> })
    }
}
