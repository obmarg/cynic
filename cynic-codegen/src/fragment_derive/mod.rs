use std::collections::{HashMap, HashSet};

use proc_macro2::{Span, TokenStream};
use quote::format_ident;

use crate::{
    attributes::{extract_meta_attrs, Attribute},
    query_dsl, FieldType, Ident, TypePath,
};

mod cynic_arguments;
mod schema_parsing;

use cynic_arguments::{arguments_from_field_attrs, FieldArgument};
use schema_parsing::{Field, Object, Schema};

pub fn fragment_derive(ast: &syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    fragment_derive_impl(ast, parse_struct_attrs(&ast.attrs)?)
}

pub fn fragment_derive_impl(
    ast: &syn::DeriveInput,
    attributes: FragmentDeriveAttributes,
) -> Result<TokenStream, syn::Error> {
    use quote::{quote, quote_spanned};

    let schema = std::fs::read_to_string(&attributes.schema_path.value).map_err(|e| {
        syn::Error::new(
            attributes.schema_path.span,
            format!("Could not load schema file: {}", e),
        )
    })?;

    let schema: Schema = graphql_parser::schema::parse_schema(&schema)
        .map_err(|e| {
            syn::Error::new(
                attributes.schema_path.span,
                format!("Could not parse schema file: {}", e),
            )
        })?
        .into();

    let object = schema
        .objects
        .get(&Ident::for_type(&attributes.graphql_type.value))
        .ok_or(syn::Error::new(
            attributes.graphql_type.span,
            format!(
                "Can't find {} in {}",
                attributes.graphql_type.value, attributes.schema_path.value
            ),
        ))?;

    let argument_struct = if let Some(arg_struct) = attributes.argument_struct {
        let arg_struct_val = Ident::new(&arg_struct.value);
        let argument_struct = quote_spanned! { arg_struct.span => #arg_struct_val };
        syn::parse2(argument_struct)?
    } else {
        syn::parse2(quote! { () })?
    };

    if let syn::Data::Struct(data_struct) = &ast.data {
        let fragment_impl = FragmentImpl::new_for(
            &data_struct,
            &ast.ident,
            &object,
            Ident::new_spanned(&attributes.query_module.value, attributes.query_module.span).into(),
            &attributes.graphql_type.value,
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

#[derive(Debug)]
pub struct FragmentDeriveAttributes {
    pub schema_path: Attribute,
    pub query_module: Attribute,
    pub graphql_type: Attribute,
    pub argument_struct: Option<Attribute>,
}

fn parse_struct_attrs(attrs: &Vec<syn::Attribute>) -> Result<FragmentDeriveAttributes, syn::Error> {
    let (mut attr_map, attr_span) = extract_meta_attrs::<DeriveAttribute>(attrs)?;

    use syn::{spanned::Spanned, Lit, Meta, MetaList, MetaNameValue, NestedMeta};

    let schema_path = attr_map
        .remove(&DeriveAttribute::SchemaPath)
        .ok_or(syn::Error::new(
            attr_span,
            "Missing required attribute: schema_path",
        ))?;

    let query_module = attr_map
        .remove(&DeriveAttribute::QueryModule)
        .ok_or(syn::Error::new(
            attr_span,
            "Missing required attribute: query_module",
        ))?;

    let graphql_type = attr_map
        .remove(&DeriveAttribute::GraphqlType)
        .ok_or(syn::Error::new(
            attr_span,
            "Missing required attribute: graphql_type",
        ))?;

    let argument_struct = attr_map.remove(&DeriveAttribute::ArgumentStruct);

    Ok(FragmentDeriveAttributes {
        schema_path,
        query_module,
        graphql_type,
        argument_struct,
    })
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum DeriveAttribute {
    SchemaPath,
    QueryModule,
    GraphqlType,
    ArgumentStruct,
}

impl std::str::FromStr for DeriveAttribute {
    type Err = String;

    fn from_str(s: &str) -> Result<DeriveAttribute, String> {
        if s == "schema_path" {
            Ok(DeriveAttribute::SchemaPath)
        } else if s == "query_module" {
            Ok(DeriveAttribute::QueryModule)
        } else if s == "graphql_type" {
            Ok(DeriveAttribute::GraphqlType)
        } else if s == "argument_struct" {
            Ok(DeriveAttribute::ArgumentStruct)
        } else {
            Err(format!("Unknown cynic attribute: {}.  Expected one of schema_path, query_module, graphql_type or argument_struct", s))
        }
    }
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
                #initial_args #field_type::fragment(args.into_args())
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
}

impl FragmentImpl {
    fn new_for(
        data_struct: &syn::DataStruct,
        name: &syn::Ident,
        object: &Object,
        query_dsl_path: TypePath,
        graphql_type_name: &str,
        argument_struct: syn::Type,
    ) -> Result<Self, syn::Error> {
        use syn::{spanned::Spanned, Fields};
        // TODO: Mostly just need to iterate over fields.
        // For first pass lets _just_ support named fields.
        // And no attributes for now.

        let target_struct = Ident::new_spanned(&name.to_string(), name.span());
        let selector_struct_path = TypePath::concat(&[
            query_dsl_path.clone(),
            object.selector_struct.clone().into(),
        ]);
        let mut fields = vec![];
        let mut constructor_params = vec![];

        if let Fields::Named(named_fields) = &data_struct.fields {
            for field in &named_fields.named {
                if let Some(ident) = &field.ident {
                    let field_name = ident.to_string();
                    constructor_params.push(ConstructorParameter {
                        name: Ident::new(&field_name),
                        type_path: field.ty.clone(),
                    });

                    let arguments = arguments_from_field_attrs(&field.attrs)?;

                    let field_name = Ident::for_field(&field_name);

                    if let Some(gql_field) = object.fields.get(&field_name) {
                        let argument_structs = argument_structs(
                            arguments,
                            gql_field,
                            &object.name,
                            &query_dsl_path,
                            field.span(),
                        )?;
                        fields.push(FieldSelectorCall {
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
                            field.span(),
                            format!(
                                "Field {} does not exist on the GraphQL type {}",
                                field_name, graphql_type_name
                            ),
                        ));
                    }
                } else {
                    return Err(syn::Error::new(
                        field.span(),
                        "QueryFragment derive currently only supports named fields",
                    ));
                }
            }
        } else {
            return Err(syn::Error::new(
                data_struct.fields.span(),
                "QueryFragment derive currently only supports named fields",
            ));
        }
        Ok(FragmentImpl {
            fields,
            target_struct,
            selector_struct_path,
            constructor_params,
            argument_struct,
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
                    use ::cynic::{QueryFragment, IntoArguments};

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
        .filter(|(name, arg)| arg.required)
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
