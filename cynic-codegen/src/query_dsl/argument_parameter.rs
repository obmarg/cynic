use crate::{Errors, FieldType, Ident, TypePath};
use proc_macro2::{Span, TokenStream};

/// A parmeter to a query_dsl function that represents a GraphQL parameter.
pub struct ArgumentParameter {
    name: Ident,
    argument_type: FieldType,
}

impl ArgumentParameter {
    pub fn new(name: Ident, argument_type: FieldType) -> ArgumentParameter {
        ArgumentParameter {
            name,
            argument_type,
        }
    }

    pub fn to_tokens(&self, path_to_markers: TypePath) -> Result<TokenStream, Errors> {
        use quote::quote;

        // TODO: Figure out if we can get rid of this generic_inner_type parameter...
        let name = &self.name;
        let type_lock = self.argument_type.to_tokens(None, path_to_markers);
        if self.argument_type.contains_enum() {
            Ok(quote! { #name: impl ::cynic::EnumArgument<#type_lock> })
        } else if self.argument_type.contains_scalar() {
            // Note that on this branch type_lock is actually just a type_spec
            // for now
            Ok(quote! { #name: #type_lock })
        } else if self.argument_type.contains_input_object() {
            Ok(quote! { #name: impl ::cynic::InputObjectArgument<#type_lock> })
        } else {
            Err(syn::Error::new(
                Span::call_site(),
                "Arguments must be scalars, enums or input objects",
            )
            .into())
        }
    }
}
