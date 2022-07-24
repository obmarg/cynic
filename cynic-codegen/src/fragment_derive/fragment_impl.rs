use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

use crate::{
    error::Errors,
    schema::types::{Field, OutputType},
    types::{self, check_spread_type, check_types_are_compatible, CheckMode},
    Ident,
};

use super::arguments::{arguments_from_field_attrs, process_arguments};
use super::fragment_derive_type::FragmentDeriveType;

use super::input::FragmentDeriveField;

pub struct FragmentImpl<'a> {
    target_struct: Ident,
    selections: Vec<Selection<'a>>,
    variables: syn::Type,
    graphql_type_name: String,
    schema_type_path: syn::Path,
}

#[allow(clippy::large_enum_variant)]
enum Selection<'a> {
    Field(FieldSelection<'a>),
    Spread(SpreadSelection),
}

struct FieldSelection<'a> {
    rust_field_type: syn::Type,
    field_marker_type_path: syn::Path,
    graphql_field_kind: FieldKind,
    graphql_field: &'a Field<'a>,
    arguments: super::arguments::Output<'a>,
    flatten: bool,
    alias: Option<String>,
    recurse_limit: Option<u8>,
    span: proc_macro2::Span,
}

struct SpreadSelection {
    rust_field_type: syn::Type,
    span: proc_macro2::Span,
}

enum FieldKind {
    Composite,
    Scalar,
    Enum,
    Interface,
    Union,
}

impl<'a> FragmentImpl<'a> {
    pub fn new_for(
        fields: &[(&FragmentDeriveField, Option<&'a Field<'a>>)],
        name: &syn::Ident,
        schema_type: &FragmentDeriveType,
        schema_module_path: &syn::Path,
        graphql_type_name: &str,
        variables: Option<&syn::Path>,
    ) -> Result<Self, Errors> {
        let target_struct = Ident::new_spanned(&name.to_string(), name.span());

        let schema_type_path = schema_type.marker_ident.to_path(schema_module_path);

        let field_module_path = schema_type.field_module.to_path(schema_module_path);

        let selections = fields
            .iter()
            .map(|(field, schema_field)| {
                process_field(
                    field,
                    *schema_field,
                    &field_module_path,
                    schema_module_path,
                    variables,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

        let variables = if let Some(vars) = variables {
            let span = vars.span();

            let variables = quote_spanned! { span => #vars };
            syn::parse2(variables)?
        } else {
            syn::parse2(quote! { () })?
        };

        Ok(FragmentImpl {
            selections,
            target_struct,
            variables,
            graphql_type_name: graphql_type_name.to_string(),
            schema_type_path,
        })
    }
}

fn process_field<'a>(
    field: &FragmentDeriveField,
    schema_field: Option<&'a Field<'a>>,
    field_module_path: &syn::Path,
    schema_module_path: &syn::Path,
    variables: Option<&syn::Path>,
) -> Result<Selection<'a>, Errors> {
    if field.type_check_mode() == CheckMode::Spreading {
        check_spread_type(&field.ty)?;

        return Ok(Selection::Spread(SpreadSelection {
            rust_field_type: field.ty.clone(),
            span: field.ty.span(),
        }));
    }

    let schema_field = schema_field.expect("only spread fields should have schema_field == None");

    let (arguments, argument_span) =
        arguments_from_field_attrs(&field.attrs)?.unwrap_or_else(|| (vec![], Span::call_site()));

    let arguments = process_arguments(
        arguments,
        schema_field,
        schema_module_path.clone(),
        variables,
        argument_span,
    )?;

    check_types_are_compatible(&schema_field.field_type, &field.ty, field.type_check_mode())?;

    let field_marker_type_path = schema_field.marker_ident().to_path(field_module_path);

    Ok(Selection::Field(FieldSelection {
        rust_field_type: field.ty.clone(),
        arguments,
        field_marker_type_path,
        graphql_field: schema_field,
        recurse_limit: field.recurse.as_ref().map(|f| **f),
        span: field.ty.span(),
        alias: field.alias(),
        graphql_field_kind: schema_field.field_type.inner_type().as_kind(),
        flatten: *field.flatten,
    }))
}

impl quote::ToTokens for FragmentImpl<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::TokenStreamExt;

        let variables = &self.variables;
        let target_struct = &self.target_struct;
        let selections = &self.selections;
        let graphql_type = proc_macro2::Literal::string(&self.graphql_type_name);
        let schema_type = &self.schema_type_path;

        tokens.append_all(quote! {
            #[automatically_derived]
            impl<'de> ::cynic::QueryFragment<'de> for #target_struct {
                type SchemaType = #schema_type;
                type Variables = #variables;

                const TYPE: Option<&'static str> = Some(#graphql_type);

                fn query(mut builder: ::cynic::queries::SelectionBuilder<Self::SchemaType, Self::Variables>) {
                    #![allow(unused_mut)]

                    #(#selections)*
                }
            }
        })
    }
}

impl quote::ToTokens for Selection<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Selection::Field(inner) => inner.to_tokens(tokens),
            Selection::Spread(inner) => inner.to_tokens(tokens),
        }
    }
}

enum SelectionMode {
    Composite,
    FlattenComposite,
    FlattenLeaf,
    Recurse(u8),
    Leaf,
}

impl quote::ToTokens for FieldSelection<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::TokenStreamExt;

        let field_marker_type_path = &self.field_marker_type_path;
        let field_type = &self.rust_field_type;
        let arguments = &self.arguments;

        let alias = self.alias.as_deref().map(|alias| {
            let alias = proc_macro2::Literal::string(alias);
            quote! {
                field_builder.alias(#alias);
            }
        });

        let selection_mode = match (&self.graphql_field_kind, self.flatten, self.recurse_limit) {
            (FieldKind::Enum | FieldKind::Scalar, true, _) => SelectionMode::FlattenLeaf,
            (FieldKind::Enum | FieldKind::Scalar, false, _) => SelectionMode::Leaf,
            (_, true, None) => SelectionMode::FlattenComposite,
            (_, false, None) => SelectionMode::Composite,
            (_, false, Some(limit)) => SelectionMode::Recurse(limit),
            _ => panic!("Uncertain how to select for this field."),
        };

        let aligned_type = match selection_mode {
            SelectionMode::Composite | SelectionMode::Leaf => {
                // If we're doing a normal select we need to align types.
                types::align_output_type(
                    &types::parsing2::parse_rust_type(field_type),
                    &self.graphql_field.field_type,
                )
                .to_syn()
            }
            _ => {
                // Recursive & flatten selections don't need types aligned
                // according to the graphql rules as they have special rules.
                field_type.clone()
            }
        };

        let schema_type_lookup = match self.graphql_field_kind {
            FieldKind::Interface | FieldKind::Composite | FieldKind::Union => {
                quote_spanned! { self.span =>
                    <#aligned_type as ::cynic::QueryFragment>::SchemaType
                }
            }
            FieldKind::Scalar => quote_spanned! { self.span =>
                <#aligned_type as ::cynic::schema::IsScalar<
                    <#field_marker_type_path as ::cynic::schema::Field>::Type
                >>::SchemaType
            },
            FieldKind::Enum => quote_spanned! { self.span =>
                <#aligned_type as ::cynic::Enum>::SchemaType
            },
        };

        tokens.append_all(match selection_mode {
            SelectionMode::Composite => {
                quote_spanned! { self.span =>
                    let mut field_builder = builder
                        .select_field::<
                            #field_marker_type_path,
                            #schema_type_lookup
                        >();

                    #alias
                    #arguments

                    <#aligned_type as ::cynic::QueryFragment>::query(
                        field_builder.select_children()
                    );
                }
            }
            SelectionMode::FlattenComposite => {
                quote_spanned! { self.span =>
                    let mut field_builder = builder
                        .select_flattened_field::<
                            #field_marker_type_path,
                            #schema_type_lookup,
                            <#field_marker_type_path as ::cynic::schema::Field>::Type,
                        >();

                    #alias
                    #arguments

                    <#aligned_type as ::cynic::QueryFragment>::query(
                        field_builder.select_children()
                    );
                }
            }
            SelectionMode::FlattenLeaf => {
                quote_spanned! { self.span =>
                    let mut field_builder = builder
                        .select_flattened_field::<
                            #field_marker_type_path,
                            #schema_type_lookup,
                            <#field_marker_type_path as ::cynic::schema::Field>::Type,
                        >();

                    #alias
                    #arguments
                }
            }
            SelectionMode::Recurse(limit) => {
                quote_spanned! { self.span =>
                    if let Some(mut field_builder) = builder
                        .recurse::<
                            #field_marker_type_path,
                            #schema_type_lookup
                        >(#limit)
                    {
                        #alias
                        #arguments

                        <#aligned_type as ::cynic::QueryFragment>::query(
                            field_builder.select_children()
                        );
                    }
                }
            }
            SelectionMode::Leaf => {
                quote_spanned! { self.span =>
                    let mut field_builder = builder
                        .select_field::<
                            #field_marker_type_path,
                            #schema_type_lookup
                        >();

                    #alias
                    #arguments
                }
            }
        });
    }
}

impl quote::ToTokens for SpreadSelection {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::TokenStreamExt;
        let field_type = &self.rust_field_type;

        tokens.append_all(quote_spanned! { self.span =>
            <#field_type as ::cynic::QueryFragment>::query(
                builder
            )
        })
    }
}

impl OutputType<'_> {
    fn as_kind(&self) -> FieldKind {
        match self {
            OutputType::Scalar(_) => FieldKind::Scalar,
            OutputType::Enum(_) => FieldKind::Enum,
            OutputType::Object(_) => FieldKind::Composite,
            OutputType::Interface(_) => FieldKind::Interface,
            OutputType::Union(_) => FieldKind::Union,
        }
    }
}
