use std::collections::HashSet;

use darling::util::SpannedValue;
use proc_macro2::{Span, TokenStream};

use crate::{query_dsl, FieldType, Ident, TypePath};

mod cynic_arguments;
mod schema_parsing;
mod type_validation;

use cynic_arguments::{arguments_from_field_attrs, FieldArgument};
use schema_parsing::{Field, Object, Schema};
use type_validation::check_types_are_compatible;

pub fn fragment_derive(ast: &syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    use darling::FromDeriveInput;

    match FragmentDeriveInput::from_derive_input(ast) {
        Ok(input) => fragment_derive_impl(input),
        Err(e) => Ok(e.write_errors()),
    }
}

pub fn fragment_derive_impl(input: FragmentDeriveInput) -> Result<TokenStream, syn::Error> {
    use quote::{quote, quote_spanned};

    let schema_path = input.schema_path;
    let schema = std::fs::read_to_string(&std::path::Path::new(&*schema_path)).map_err(|e| {
        syn::Error::new(
            schema_path.span(),
            format!("Could not load schema file: {}", e),
        )
    })?;

    let schema: Schema = graphql_parser::schema::parse_schema(&schema)
        .map_err(|e| {
            syn::Error::new(
                schema_path.span(),
                format!("Could not parse schema file: {}", e),
            )
        })?
        .into();

    let graphql_type = input.graphql_type;
    let object = schema
        .objects
        .get(&Ident::for_type(&*graphql_type))
        .ok_or(syn::Error::new(
            graphql_type.span(),
            format!("Can't find {} in {}", *graphql_type, *schema_path),
        ))?;

    let argument_struct = if let Some(arg_struct) = input.argument_struct {
        let span = arg_struct.span();

        let arg_struct_val: Ident = arg_struct.into();
        let argument_struct = quote_spanned! { span => #arg_struct_val };
        syn::parse2(argument_struct)?
    } else {
        syn::parse2(quote! { () })?
    };

    if let darling::ast::Data::Struct(fields) = &input.data {
        let query_module = input.query_module;
        let fragment_impl = FragmentImpl::new_for(
            &fields,
            &input.ident,
            &object,
            Ident::new_spanned(&*query_module, query_module.span()).into(),
            &graphql_type,
            argument_struct,
        )?;
        Ok(quote::quote! {
            #fragment_impl
        })
    } else {
        Err(syn::Error::new(
            Span::call_site(),
            format!("QueryFragment can only be derived from a struct"),
        ))
    }
}

#[derive(darling::FromDeriveInput)]
#[darling(attributes(cynic), supports(struct_named))]
pub struct FragmentDeriveInput {
    ident: proc_macro2::Ident,
    data: darling::ast::Data<(), FragmentDeriveField>,

    pub schema_path: SpannedValue<String>,
    pub query_module: SpannedValue<String>,
    pub graphql_type: SpannedValue<String>,
    #[darling(default)]
    pub argument_struct: Option<syn::Ident>,
}

#[derive(darling::FromField)]
#[darling(attributes(cynic), forward_attrs(cynic_arguments))]
struct FragmentDeriveField {
    ident: Option<proc_macro2::Ident>,
    ty: syn::Type,
    attrs: Vec<syn::Attribute>,
}

struct FieldSelectorParameter {
    field_type: syn::Type,
}

impl quote::ToTokens for FieldSelectorParameter {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let field_type = &self.field_type;

        tokens.append_all(quote! {
            #field_type::fragment()
        });
    }
}

enum SelectorFunction {
    Field(TypePath),
    Opt(Box<SelectorFunction>),
    Vector(Box<SelectorFunction>),
}

impl SelectorFunction {
    fn for_field(field_type: &FieldType, field_constructor: TypePath) -> SelectorFunction {
        if let FieldType::Scalar(_, _) = field_type {
            // We special case scalars as their vec/optional-ness is always handled
            // by the functions on the generated query_dsl.
            // Whereas other types call into the QueryFragment::query function
            // which can't know whether the type is optional/repeated at this level.
            return SelectorFunction::Field(field_constructor);
        }

        if field_type.is_nullable() {
            SelectorFunction::Opt(Box::new(SelectorFunction::for_field(
                &field_type.clone().as_required(),
                field_constructor,
            )))
        } else if let FieldType::List(inner, _) = field_type {
            SelectorFunction::Vector(Box::new(SelectorFunction::for_field(
                &inner,
                field_constructor,
            )))
        } else {
            SelectorFunction::Field(field_constructor)
        }
    }
}

impl SelectorFunction {
    fn to_call(&self, parameters: TokenStream) -> TokenStream {
        use quote::quote;

        match self {
            SelectorFunction::Field(type_path) => quote! { #type_path(#parameters) },
            SelectorFunction::Opt(inner) => inner.to_call(parameters),
            SelectorFunction::Vector(inner) => inner.to_call(parameters),
        }
    }
}

struct FieldSelectorCall {
    selector_function: SelectorFunction,
    contains_composite: bool,
    query_fragment_field_type: syn::Type,
    argument_structs: Vec<ArgumentStruct>,
}

impl quote::ToTokens for FieldSelectorCall {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let initial_args = if self.argument_structs.is_empty() {
            quote! {}
        } else {
            let argument_structs = &self.argument_structs;
            quote! {
                #(#argument_structs),* ,
            }
        };

        let inner_call = if self.contains_composite {
            let field_type = &self.query_fragment_field_type;
            quote! {
                #initial_args #field_type::fragment(FromArguments::from_arguments(&args))
            }
        } else {
            quote! {#initial_args}
        };

        let selector_function_call = &self.selector_function.to_call(inner_call);

        tokens.append_all(quote! {
            #selector_function_call
        });
    }
}

struct ArgumentStruct {
    type_name: TypePath,
    fields: Vec<FieldArgument>,
    required: bool,
}

impl quote::ToTokens for ArgumentStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let type_name = &self.type_name;
        let arguments = &self.fields;
        let default = if !self.required {
            quote! { ..Default::default() }
        } else {
            quote! {}
        };

        tokens.append_all(quote! {
            #type_name {
                #(#arguments, )*
                #default
            }
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
        query_dsl_path: TypePath,
        graphql_type_name: &str,
        argument_struct: syn::Type,
    ) -> Result<Self, syn::Error> {
        let target_struct = Ident::new_spanned(&name.to_string(), name.span());
        let selector_struct_path = TypePath::concat(&[
            query_dsl_path.clone(),
            object.selector_struct.clone().into(),
        ]);
        let mut field_selectors = vec![];
        let mut constructor_params = vec![];

        if fields.style != darling::ast::Style::Struct {
            return Err(syn::Error::new(
                name.span(),
                "QueryFragment derive currently only supports named fields",
            ));
        }

        for field in &fields.fields {
            if let Some(ident) = &field.ident {
                let field_name = ident.to_string();
                constructor_params.push(ConstructorParameter {
                    name: Ident::new(&field_name),
                    type_path: field.ty.clone(),
                });

                let arguments = arguments_from_field_attrs(&field.attrs)?;

                let field_name = Ident::for_field(&field_name);

                if let Some(gql_field) = object.fields.get(&field_name) {
                    check_types_are_compatible(&gql_field.field_type, &field.ty)?;

                    let argument_structs = argument_structs(
                        arguments,
                        gql_field,
                        &object.name,
                        &query_dsl_path,
                        ident.span(),
                    )?;
                    field_selectors.push(FieldSelectorCall {
                        selector_function: SelectorFunction::for_field(
                            &gql_field.field_type,
                            TypePath::concat(&[
                                selector_struct_path.clone(),
                                field_name.clone().into(),
                            ]),
                        ),
                        contains_composite: !gql_field.field_type.contains_scalar(),
                        query_fragment_field_type: gql_field
                            .field_type
                            .get_inner_type_from_syn(&field.ty),
                        argument_structs,
                    })
                } else {
                    return Err(syn::Error::new(
                        ident.span(),
                        format!(
                            "Field {} does not exist on the GraphQL type {}",
                            field_name, graphql_type_name
                        ),
                    ));
                }
            }
        }

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
            impl ::cynic::QueryFragment for #target_struct {
                type SelectionSet = ::cynic::SelectionSet<'static, Self, #selector_struct>;
                type Arguments = #argument_struct;

                fn fragment(args: Self::Arguments) -> Self::SelectionSet {
                    use ::cynic::{QueryFragment, FromArguments};

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

/// Constructs some ArgumentStructs from the arguments of a
fn argument_structs(
    arguments: Vec<FieldArgument>,
    field: &Field,
    containing_object_name: &Ident,
    query_dsl_path: &TypePath,
    missing_arg_span: Span,
) -> Result<Vec<ArgumentStruct>, syn::Error> {
    let all_required: HashSet<Ident> = field
        .arguments
        .iter()
        .filter(|(_name, arg)| arg.required)
        .map(|(name, _)| name.clone())
        .collect();

    let provided_names: HashSet<Ident> = arguments
        .iter()
        .map(|arg| arg.argument_name.clone().into())
        .collect();

    let missing_args: Vec<_> = all_required
        .difference(&provided_names)
        .map(|s| s.to_string())
        .collect();
    if !missing_args.is_empty() {
        let missing_args = missing_args.join(", ");

        return Err(syn::Error::new(
            missing_arg_span,
            format!("Missing cynic_arguments: {}", missing_args),
        ));
    }

    let mut required = vec![];
    let mut optional = vec![];
    for provided_argument in arguments {
        let arg_name = provided_argument.argument_name.clone().into();
        if let Some(argument_def) = field.arguments.get(&arg_name) {
            if argument_def.required {
                required.push(provided_argument);
            } else {
                optional.push(provided_argument);
            }
        } else {
            return Err(syn::Error::new(
                provided_argument.argument_name.span(),
                format!(
                    "{} is not a valid argument for this field",
                    provided_argument.argument_name
                ),
            ));
        }
    }

    let mut rv = vec![];
    if field.arguments.iter().any(|(_, arg)| arg.required) {
        rv.push(ArgumentStruct {
            type_name: TypePath::concat(&[
                query_dsl_path.clone(),
                Ident::for_module(&containing_object_name.to_string()).into(),
                query_dsl::ArgumentStruct::name_for_field(&field.name.to_string(), true).into(),
            ]),
            fields: required,
            required: true,
        });
    }

    if field.arguments.iter().any(|(_, arg)| !arg.required) {
        rv.push(ArgumentStruct {
            type_name: TypePath::concat(&[
                query_dsl_path.clone(),
                Ident::for_module(&containing_object_name.to_string()).into(),
                query_dsl::ArgumentStruct::name_for_field(&field.name.to_string(), false).into(),
            ]),
            fields: optional,
            required: false,
        });
    }

    Ok(rv)
}
