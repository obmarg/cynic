use std::collections::{HashMap, HashSet};

use proc_macro2::{Span, TokenStream};
use syn::spanned::Spanned;

use crate::{
    load_schema,
    type_validation::{check_spread_type, check_types_are_compatible, CheckMode},
    Errors, FieldType, Ident, TypePath,
};

mod arguments;
mod schema_parsing;
mod type_ext;

pub(crate) mod input;

use arguments::{arguments_from_field_attrs, FieldArgument};
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

    input.validate()?;

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

    if let darling::ast::Data::Struct(fields) = &input.data {
        let graphql_name = &(input.graphql_type_name());
        let schema_module = input.schema_module();
        let fragment_impl = FragmentImpl::new_for(
            &fields,
            &input.ident,
            &object,
            Ident::new_spanned(&*schema_module, schema_module.span()).into(),
            graphql_name,
            argument_struct,
        )?;
        Ok(quote::quote! {
            #fragment_impl
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
    Field(TypePath),
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
        field_constructor: TypePath,
        flatten: bool,
        recurse_limit: Option<u8>,
    ) -> FieldTypeSelectorCall {
        if flatten {
            FieldTypeSelectorCall::Flatten(Box::new(FieldTypeSelectorCall::for_field(
                field_type,
                field_constructor,
                false,
                None,
            )))
        } else if let Some(limit) = recurse_limit {
            let inner_selector = Box::new(FieldTypeSelectorCall::for_field(
                field_type,
                field_constructor,
                false,
                None,
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
            )))
        } else if let FieldType::List(inner, _) = field_type {
            FieldTypeSelectorCall::Vector(Box::new(FieldTypeSelectorCall::for_field(
                &inner,
                field_constructor,
                false,
                None,
            )))
        } else {
            FieldTypeSelectorCall::Field(field_constructor)
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

struct FragmentImpl {
    target_struct: Ident,
    fields: Vec<FieldSelectorCall>,
    selector_struct_path: TypePath,
    constructor_params: Vec<ConstructorParameter>,
    argument_struct: syn::Type,
    graphql_type_name: String,
}

impl FragmentImpl {
    fn new_for(
        fields: &darling::ast::Fields<FragmentDeriveField>,
        name: &syn::Ident,
        object: &Object,
        schema_module_path: TypePath,
        graphql_type_name: &str,
        argument_struct: syn::Type,
    ) -> Result<Self, syn::Error> {
        let target_struct = Ident::new_spanned(&name.to_string(), name.span());
        let selector_struct_path =
            TypePath::concat(&[schema_module_path, object.selector_struct.clone().into()]);

        if fields.style != darling::ast::Style::Struct {
            return Err(syn::Error::new(
                name.span(),
                "QueryFragment derive currently only supports named fields",
            ));
        }

        let (constructor_params, field_selectors) = fields
            .fields
            .iter()
            .map(|field| process_field(field, object, &selector_struct_path, graphql_type_name))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .unzip();

        Ok(FragmentImpl {
            fields: field_selectors,
            target_struct,
            selector_struct_path,
            constructor_params,
            argument_struct,
            graphql_type_name: graphql_type_name.to_string(),
        })
    }
}

fn process_field(
    field: &FragmentDeriveField,
    object: &Object,
    selector_struct_path: &TypePath,
    graphql_type_name: &str,
) -> Result<(ConstructorParameter, FieldSelectorCall), syn::Error> {
    // Should be safe to unwrap because we've already checked we have a struct
    // style input
    let (field_ident, graphql_ident) = field.ident.as_ref().zip(field.graphql_ident()).unwrap();

    let field_name_span = graphql_ident.span();

    let constructor_param = ConstructorParameter {
        name: field_ident.clone().into(),
        type_path: field.ty.clone(),
    };

    let arguments = arguments_from_field_attrs(&field.attrs)?;

    if field.type_check_mode() == CheckMode::Spreading {
        check_spread_type(&field.ty)?;

        let field_selector = FieldSelectorCall {
            selector_function: FieldTypeSelectorCall::for_spread(),
            style: NamedTypeSelectorStyle::QueryFragment(field.ty.clone()),
            required_arguments: vec![],
            optional_arguments: vec![],
            recurse_limit: None,
            span: field.ty.span(),
        };

        Ok((constructor_param, field_selector))
    } else if let Some(gql_field) = object.fields.get(&graphql_ident) {
        check_types_are_compatible(&gql_field.field_type, &field.ty, field.type_check_mode())?;

        let (required_arguments, optional_arguments) =
            validate_and_group_args(arguments, gql_field, field_name_span)?;

        let field_selector = FieldSelectorCall {
            selector_function: FieldTypeSelectorCall::for_field(
                &gql_field.field_type,
                TypePath::concat(&[selector_struct_path.clone(), graphql_ident.clone().into()]),
                *field.flatten,
                field.recurse.as_ref().map(|f| **f),
            ),
            style: if gql_field.field_type.contains_scalar() {
                NamedTypeSelectorStyle::Scalar
            } else if gql_field.field_type.contains_enum() {
                NamedTypeSelectorStyle::Enum(field.ty.inner_type())
            } else {
                NamedTypeSelectorStyle::QueryFragment(field.ty.inner_type())
            },
            required_arguments,
            optional_arguments,
            recurse_limit: field.recurse.as_ref().map(|limit| **limit),
            span: field.ty.span(),
        };

        Ok((constructor_param, field_selector))
    } else {
        let candidates = object.fields.keys().map(|k| k.graphql_name());
        let graphql_name = graphql_ident.graphql_name();
        let guess_value = guess_field(candidates, &graphql_name);
        Err(syn::Error::new(
            field_name_span,
            format!(
                "Field {} does not exist on the GraphQL type {}.{}",
                graphql_name,
                graphql_type_name,
                format_guess(guess_value).as_str()
            ),
        ))
    }
}

impl quote::ToTokens for FragmentImpl {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let argument_struct = &self.argument_struct;
        let target_struct = &self.target_struct;
        let selector_struct = &self.selector_struct_path;
        let fields = &self.fields;
        let constructor_params = &self.constructor_params;
        let graphql_type = proc_macro2::Literal::string(&self.graphql_type_name);
        let constructor_param_names = self
            .constructor_params
            .iter()
            .map(|p| &p.name)
            .collect::<Vec<_>>();

        let map_function = quote::format_ident!("map{}", fields.len());

        tokens.append_all(quote! {
            #[automatically_derived]
            impl ::cynic::QueryFragment for #target_struct {
                type SelectionSet = ::cynic::SelectionSet<'static, Self, #selector_struct>;
                type Arguments = #argument_struct;

                fn fragment(context: ::cynic::FragmentContext<Self::Arguments>) -> Self::SelectionSet {
                    use ::cynic::{QueryFragment, FromArguments, Enum};

                    let args = context.args;

                    let new = |#(#constructor_params),*| #target_struct {
                        #(#constructor_param_names),*
                    };

                    ::cynic::selection_set::#map_function(
                        new,
                        #(
                            #fields
                        ),*
                    )
                }

                fn graphql_type() -> String {
                    #graphql_type.to_string()
                }
            }
        })
    }
}

/// Validates the FieldArguments against the arguments defined on field
/// in the schema.  If everythings good, groups into required & optional
/// arguments in the correct order.
fn validate_and_group_args(
    arguments: Vec<FieldArgument>,
    field: &Field,
    missing_arg_span: Span,
) -> Result<(Vec<FieldArgument>, Vec<FieldArgument>), syn::Error> {
    let all_required: HashSet<Ident> = field
        .arguments
        .iter()
        .filter(|arg| arg.required)
        .map(|arg| arg.name.clone())
        .collect();

    let provided_names: HashSet<Ident> = arguments
        .iter()
        .map(|arg| arg.argument_name.clone().into())
        .collect();

    let missing_args: Vec<_> = all_required
        .difference(&provided_names)
        .map(|s| s.graphql_name())
        .collect();
    if !missing_args.is_empty() {
        let missing_args = missing_args.join(", ");

        return Err(syn::Error::new(
            missing_arg_span,
            format!("Missing arguments: {}", missing_args),
        ));
    }

    let all_arg_names: HashSet<Ident> =
        field.arguments.iter().map(|arg| arg.name.clone()).collect();

    let unknown_args: Vec<_> = provided_names
        .difference(&all_arg_names)
        .map(|s| s.graphql_name())
        .collect();

    if !unknown_args.is_empty() {
        let unknown_args = unknown_args.join(", ");

        return Err(syn::Error::new(
            missing_arg_span,
            format!("Unknown arguments: {}", unknown_args),
        ));
    }

    let provided_arguments: HashMap<Ident, _> = arguments
        .into_iter()
        .map(|a| (a.argument_name.clone().into(), a))
        .collect();

    let mut required = vec![];
    let mut optionals = vec![];
    for schema_arg in &field.arguments {
        if let Some(provided_arg) = provided_arguments.get(&schema_arg.name) {
            if schema_arg.required {
                required.push(provided_arg);
            } else {
                optionals.push(provided_arg);
            }
        }
    }

    Ok((
        required.into_iter().cloned().collect(),
        optionals.into_iter().cloned().collect(),
    ))
}
