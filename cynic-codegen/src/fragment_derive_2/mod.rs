use std::collections::{HashMap, HashSet};

use proc_macro2::{Span, TokenStream};
use syn::spanned::Spanned;

use crate::{
    fragment_derive_2::deserialize_impl::DeserializeImpl,
    ident::PathExt,
    load_schema,
    type_validation::{check_spread_type, check_types_are_compatible, CheckMode},
    Errors, FieldType, Ident,
};

mod arguments;
mod deserialize_impl;
mod fragment_impl;
mod schema_parsing;
mod type_ext;

pub(crate) mod input;

use arguments::{arguments_from_field_attrs, FieldArgument};
use fragment_impl::FragmentImpl;
use schema_parsing::{Field, Object};
use type_ext::SynTypeExt;

pub use input::{FragmentDeriveField, FragmentDeriveInput};

use crate::suggestions::{format_guess, guess_field};
pub(crate) use schema_parsing::Schema;

pub fn fragment_derive(ast: &syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    use darling::FromDeriveInput;

    match FragmentDeriveInput::from_derive_input(ast) {
        Ok(input) => load_schema(&*input.schema_path)
            .map_err(|e| Errors::from(e.into_syn_error(input.schema_path.span())))
            .map(Schema::from)
            .and_then(|schema| fragment_derive_impl(input, &schema))
            .or_else(|e| Ok(e.to_compile_errors())),
        Err(e) => Ok(e.write_errors()),
    }
}

pub fn fragment_derive_impl(
    input: FragmentDeriveInput,
    schema: &Schema,
) -> Result<TokenStream, Errors> {
    use quote::{quote, quote_spanned};

    let mut input = input;
    input.validate()?;
    input.detect_aliases();

    let schema_path = &input.schema_path;

    let object = schema
        .objects
        .get(&Ident::for_type(&input.graphql_type_name()))
        .ok_or_else(|| {
            syn::Error::new(
                input.graphql_type_span(),
                format!(
                    "Can't find {} in {}",
                    input.graphql_type_name(),
                    **schema_path
                ),
            )
        })?;

    let input_argument_struct = (&input.argument_struct).clone();
    let argument_struct = if let Some(arg_struct) = input_argument_struct {
        let span = arg_struct.span();

        let arg_struct_val: Ident = arg_struct.into();
        let argument_struct = quote_spanned! { span => #arg_struct_val };
        syn::parse2(argument_struct)?
    } else {
        syn::parse2(quote! { () })?
    };

    let graphql_name = &(input.graphql_type_name());
    let schema_module = input.schema_module();
    let ident = input.ident;
    if let darling::ast::Data::Struct(fields) = input.data {
        let fragment_impl = FragmentImpl::new_for(
            &fields,
            &ident,
            object,
            schema_module,
            graphql_name,
            argument_struct,
        )?;

        let deserialize_impl = DeserializeImpl::new(&fields, &ident);

        Ok(quote::quote! {
            #fragment_impl
            #deserialize_impl
        })
    } else {
        Err(syn::Error::new(
            Span::call_site(),
            "QueryFragment can only be derived from a struct".to_string(),
        )
        .into())
    }
}

/// Selector for a "field type" - i.e. a nullable/list/required type that
/// references some named schema type.
enum FieldTypeSelectorCall {
    Field(syn::Path),
    AliasedField(String, syn::Path),
    Opt(Box<FieldTypeSelectorCall>),
    Vector(Box<FieldTypeSelectorCall>),
    Flatten(Box<FieldTypeSelectorCall>),
    Recurse(u8, Box<FieldTypeSelectorCall>, bool),
    Box(Box<FieldTypeSelectorCall>),
    Spread,
}

impl FieldTypeSelectorCall {
    fn for_spread() -> FieldTypeSelectorCall {
        FieldTypeSelectorCall::Spread
    }

    fn for_field(
        field_type: &FieldType,
        field_constructor: syn::Path,
        flatten: bool,
        recurse_limit: Option<u8>,
        alias: Option<String>,
    ) -> FieldTypeSelectorCall {
        if flatten {
            FieldTypeSelectorCall::Flatten(Box::new(FieldTypeSelectorCall::for_field(
                field_type,
                field_constructor,
                false,
                None,
                alias,
            )))
        } else if let Some(limit) = recurse_limit {
            let inner_selector = Box::new(FieldTypeSelectorCall::for_field(
                field_type,
                field_constructor,
                false,
                None,
                alias,
            ));

            if field_type.is_list() {
                // List types can just recurse - no need for boxes
                FieldTypeSelectorCall::Recurse(limit, inner_selector, field_type.is_nullable())
            } else if field_type.is_nullable() {
                // Optional types need to be wrapped in Box to keep the rust compiler happy
                // i.e. `Box<Option<T>>`
                FieldTypeSelectorCall::Box(Box::new(FieldTypeSelectorCall::Recurse(
                    limit,
                    inner_selector,
                    field_type.is_nullable(),
                )))
            } else {
                // Required types need their inner types to be wrapped in box
                // i.e. `Option<Box<T>>`
                FieldTypeSelectorCall::Recurse(
                    limit,
                    Box::new(FieldTypeSelectorCall::Box(inner_selector)),
                    field_type.is_nullable(),
                )
            }
        } else if field_type.is_nullable() {
            FieldTypeSelectorCall::Opt(Box::new(FieldTypeSelectorCall::for_field(
                &field_type.clone().as_required(),
                field_constructor,
                false,
                None,
                alias,
            )))
        } else if let FieldType::List(inner, _) = field_type {
            FieldTypeSelectorCall::Vector(Box::new(FieldTypeSelectorCall::for_field(
                inner,
                field_constructor,
                false,
                None,
                alias,
            )))
        } else {
            match alias {
                Some(alias) => FieldTypeSelectorCall::AliasedField(alias, field_constructor),
                None => FieldTypeSelectorCall::Field(field_constructor),
            }
        }
    }
}

/// The call style to use for a particular named type selector function
enum NamedTypeSelectorStyle {
    QueryFragment(syn::Type),
    Enum(syn::Type),
    Scalar,
}

struct FieldSelectorCall {
    selector_function: FieldTypeSelectorCall,
    style: NamedTypeSelectorStyle,
    required_arguments: Vec<FieldArgument>,
    optional_arguments: Vec<FieldArgument>,
    recurse_limit: Option<u8>,
    span: proc_macro2::Span,
}

struct ConstructorParameter {
    name: Ident,
    type_path: syn::Type,
}

impl quote::ToTokens for ConstructorParameter {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let name = &self.name;
        let type_path = &self.type_path;

        tokens.append_all(quote! {
            #name: #type_path
        })
    }
}

#[cfg(test)]
mod tests;
