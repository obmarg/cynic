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

impl FieldTypeSelectorCall {
    fn to_call(
        &self,
        required_arguments: &[FieldArgument],
        optional_arguments: &[FieldArgument],
        inner_selection_tokens: TokenStream,
    ) -> TokenStream {
        use quote::quote;

        match self {
            FieldTypeSelectorCall::Field(type_path) => {
                let required_arguments = required_arguments.iter().map(|arg| &arg.expr);
                let optional_arg_names = optional_arguments.iter().map(|arg| &arg.argument_name);
                let optional_arg_exprs = optional_arguments.iter().map(|arg| &arg.expr);

                quote! {
                    #type_path(
                        #(#required_arguments, )*
                    )
                    #(
                        .#optional_arg_names(#optional_arg_exprs)
                    )*
                    .select(#inner_selection_tokens)
                }
            }
            FieldTypeSelectorCall::AliasedField(alias, type_path) => {
                let required_arguments = required_arguments.iter().map(|arg| &arg.expr);
                let optional_arg_names = optional_arguments.iter().map(|arg| &arg.argument_name);
                let optional_arg_exprs = optional_arguments.iter().map(|arg| &arg.expr);
                let alias = proc_macro2::Literal::string(alias);

                quote! {
                    #type_path(
                        #(#required_arguments, )*
                    )
                    #(
                        .#optional_arg_names(#optional_arg_exprs)
                    )*
                    .select_aliased(#alias, #inner_selection_tokens)
                }
            }
            FieldTypeSelectorCall::Opt(inner) => inner.to_call(
                required_arguments,
                optional_arguments,
                inner_selection_tokens,
            ),
            FieldTypeSelectorCall::Vector(inner) => inner.to_call(
                required_arguments,
                optional_arguments,
                inner_selection_tokens,
            ),
            FieldTypeSelectorCall::Flatten(inner) => {
                let inner_call = inner.to_call(
                    required_arguments,
                    optional_arguments,
                    inner_selection_tokens,
                );

                quote! {
                    #inner_call.map(|item| {
                        use ::cynic::utils::FlattenInto;
                        item.flatten_into()
                    })
                }
            }
            FieldTypeSelectorCall::Recurse(limit, inner, field_nullable) => {
                let inner_call = inner.to_call(
                    required_arguments,
                    optional_arguments,
                    inner_selection_tokens,
                );

                let recurse_branch = if !field_nullable {
                    // If the field is required we need to wrap it in an option
                    quote! {
                        #inner_call.map(Some)
                    }
                } else {
                    // Otherwise, just let the fields option do the work.
                    inner_call
                };

                quote! {
                    if context.recurse_depth != Some(#limit) {
                        #recurse_branch
                    } else {
                        ::cynic::selection_set::succeed_using(|| None)
                    }
                }
            }
            FieldTypeSelectorCall::Box(inner) => {
                let inner_call = inner.to_call(
                    required_arguments,
                    optional_arguments,
                    inner_selection_tokens,
                );

                quote! {
                    #inner_call.map(Box::new)
                }
            }
            FieldTypeSelectorCall::Spread => {
                quote! { #inner_selection_tokens }
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

impl quote::ToTokens for FieldSelectorCall {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, quote_spanned, TokenStreamExt};

        let span = self.span;

        let inner_selection_tokens = match (&self.style, self.recurse_limit) {
            (NamedTypeSelectorStyle::Scalar, _) => {
                quote_spanned! {span => ::cynic::selection_set::scalar()}
            }
            (NamedTypeSelectorStyle::Enum(enum_type), _) => quote_spanned! {span =>
                #enum_type::select()
            },
            (NamedTypeSelectorStyle::QueryFragment(field_type), None) => quote_spanned! {span =>
                #field_type::fragment(context.with_args(FromArguments::from_arguments(args)))
            },
            (NamedTypeSelectorStyle::QueryFragment(field_type), Some(_)) => quote_spanned! {span =>
                #field_type::fragment(context.recurse().with_args(FromArguments::from_arguments(args)))
            },
        };

        let selector_function_call = &self.selector_function.to_call(
            &self.required_arguments,
            &self.optional_arguments,
            inner_selection_tokens,
        );

        tokens.append_all(quote! {
            #selector_function_call
        });
    }
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
