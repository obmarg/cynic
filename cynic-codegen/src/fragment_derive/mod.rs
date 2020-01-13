use std::collections::HashMap;

use proc_macro2::{Span, TokenStream};
use quote::format_ident;

use crate::query_dsl::{FieldSelector, QueryDsl, SelectorStruct};
use crate::{FieldType, Ident, TypePath};

pub fn fragment_derive(ast: &syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    let struct_attrs = parse_struct_attrs(&ast.attrs)?;

    let schema = std::fs::read_to_string(&struct_attrs.schema_path.value).map_err(|e| {
        syn::Error::new(
            struct_attrs.schema_path.span,
            format!("Could not load schema file: {}", e),
        )
    })?;

    let query_dsl: QueryDsl = graphql_parser::schema::parse_schema(&schema)
        .map_err(|e| {
            syn::Error::new(
                struct_attrs.schema_path.span,
                format!("Could not parse schema: {}", e),
            )
        })?
        .into();

    let selector_struct_name = Ident::for_type(&struct_attrs.graphql_type.value);
    let selector_struct = query_dsl
        .selectors
        .iter()
        .find(|s| s.name == selector_struct_name)
        .ok_or(syn::Error::new(
            struct_attrs.graphql_type.span,
            format!(
                "Can't find {} in {}",
                struct_attrs.graphql_type.value, struct_attrs.schema_path.value
            ),
        ))?;

    if let syn::Data::Struct(data_struct) = &ast.data {
        let fragment_impl = FragmentImpl::new_for(
            &data_struct,
            &ast.ident,
            &selector_struct,
            Ident::new_spanned(
                &struct_attrs.query_module.value,
                struct_attrs.query_module.span,
            )
            .into(),
            &struct_attrs.graphql_type.value,
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
struct Attribute {
    value: String,
    span: Span,
}

impl From<syn::LitStr> for Attribute {
    fn from(s: syn::LitStr) -> Self {
        Attribute {
            value: s.value(),
            span: s.span(),
        }
    }
}

#[derive(Debug)]
struct CynicAttributes {
    schema_path: Attribute,
    query_module: Attribute,
    graphql_type: Attribute,
}

fn parse_struct_attrs(attrs: &Vec<syn::Attribute>) -> Result<CynicAttributes, syn::Error> {
    use syn::{spanned::Spanned, Lit, Meta, MetaList, MetaNameValue, NestedMeta};

    let cynic_meta = attrs
        .iter()
        .find(|a| a.path.is_ident(&format_ident!("cynic")))
        .ok_or(syn::Error::new(
            Span::call_site(),
            "cynic attribute not provided",
        ))
        .and_then(|attr| attr.parse_meta())?;

    let mut attr_map: HashMap<DeriveAttribute, Attribute> = HashMap::new();

    if let Meta::List(MetaList { nested, .. }) = &cynic_meta {
        for meta in nested {
            if let NestedMeta::Meta(Meta::NameValue(MetaNameValue { path, lit, .. })) = meta {
                if let Some(ident) = path.get_ident() {
                    let attr_name = ident
                        .to_string()
                        .parse()
                        .map_err(|e| syn::Error::new(ident.span(), &e))?;

                    if let Lit::Str(lit_str) = lit {
                        attr_map.insert(attr_name, lit_str.clone().into());
                    } else {
                        // TODO: Re-factor this into something nicer...
                        // Could probably return an Error enum and move the strings
                        // elsewhere.
                        // Could potentially also do this with combinators or similar..
                        return Err(syn::Error::new(
                            lit.span(),
                            "values in the cynic attribute should be string literals",
                        ));
                    }
                } else {
                    return Err(syn::Error::new(
                        path.span(),
                        "keys in the cynic attribute should be a single identifier",
                    ));
                }
            } else {
                return Err(syn::Error::new(
                    meta.span(),
                    "The cynic attribute accepts a list of key=\"value\" pairs",
                ));
            }
        }
    } else {
        return Err(syn::Error::new(
            cynic_meta.span(),
            "The cynic attribute accepts a list of key=\"value\" pairs",
        ));
    }

    let schema_path = attr_map
        .remove(&DeriveAttribute::SchemaPath)
        .ok_or(syn::Error::new(
            cynic_meta.span(),
            "Missing required attribute: schema_path",
        ))?;

    let query_module = attr_map
        .remove(&DeriveAttribute::QueryModule)
        .ok_or(syn::Error::new(
            cynic_meta.span(),
            "Missing required attribute: query_module",
        ))?;

    let graphql_type = attr_map
        .remove(&DeriveAttribute::GraphqlType)
        .ok_or(syn::Error::new(
            cynic_meta.span(),
            "Missing required attribute: graphql_type",
        ))?;

    Ok(CynicAttributes {
        schema_path,
        query_module,
        graphql_type,
    })
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum DeriveAttribute {
    SchemaPath,
    QueryModule,
    GraphqlType,
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
        } else {
            Err(format!("Unknown cynic attribute: {}.  Expected one of schema_path, query_module, or graphql_type", s))
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
            #field_type::selection_set()
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
            SelectorFunction::Opt(inner) => {
                let inner = inner.to_call(parameters);
                quote! {
                    ::cynic::selection_set::option(#inner)
                }
            }
            SelectorFunction::Vector(inner) => {
                let inner = inner.to_call(parameters);
                quote! {
                   ::cynic::selection_set::vec(#inner)
                }
            }
        }
    }
}

struct FieldSelectorCall {
    selector_function: SelectorFunction,
    // TODO: not sure parameters is even needed - remove it if not...
    parameters: Vec<FieldSelectorParameter>,
}

impl quote::ToTokens for FieldSelectorCall {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let parameters = &self.parameters;
        // TODO: If to_call doesn't require parameters we could convert it into ToTokens
        let selector_function_call = &self.selector_function.to_call(quote! {});

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
}

impl FragmentImpl {
    fn new_for(
        data_struct: &syn::DataStruct,
        name: &syn::Ident,
        selector_struct: &SelectorStruct,
        query_dsl_path: TypePath,
        graphql_type_name: &str,
    ) -> Result<Self, syn::Error> {
        use syn::{spanned::Spanned, Fields};
        // TODO: Mostly just need to iterate over fields.
        // For first pass lets _just_ support named fields.
        // And no attributes for now.
        let selector_fields: HashMap<String, &FieldSelector> = selector_struct
            .fields
            .iter()
            .map(|f| (f.rust_field_name.to_string(), f))
            .collect();

        let target_struct = Ident::new_spanned(&name.to_string(), name.span());
        let selector_struct_path =
            TypePath::concat(&[query_dsl_path, selector_struct.name.clone().into()]);
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

                    if let Some(&selector) = selector_fields.get(&field_name) {
                        fields.push(FieldSelectorCall {
                            selector_function: SelectorFunction::for_field(
                                &selector.field_type,
                                TypePath::concat(&[
                                    selector_struct_path.clone(),
                                    selector.rust_field_name.clone().into(),
                                ]),
                            ),
                            parameters: if selector.field_type.contains_scalar() {
                                vec![]
                            } else {
                                vec![FieldSelectorParameter {
                                    // TODO: This bit is wrong.  If this is an Option<Vec> then we can't just
                                    // call selection_set on it...
                                    field_type: field.ty.clone(),
                                }]
                            },
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
        })
    }
}

impl quote::ToTokens for FragmentImpl {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

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
            impl ::cynic::QueryFragment<'static> for #target_struct {
                type SelectionSet = ::cynic::SelectionSet<'static, Self, #selector_struct>;

                fn selection_set() -> Self::SelectionSet {
                    use ::cynic::QueryFragment;

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
