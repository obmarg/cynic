use {
    proc_macro2::{Span, TokenStream},
    quote::{quote, quote_spanned},
    syn::spanned::Spanned,
};

use crate::{
    error::Errors,
    schema::{
        types::{Field, OutputType},
        Schema, Unvalidated,
    },
    types::{self, check_spread_type, check_types_are_compatible, CheckMode},
    variables_fields_path,
};

use super::{
    arguments::{arguments_from_field_attrs, process_arguments},
    directives::{process_directive, AnalysedFieldDirective},
    fragment_derive_type::FragmentDeriveType,
    FieldKind,
};

use super::input::FragmentDeriveField;

pub struct FragmentImpl<'schema, 'a> {
    target_struct: &'a proc_macro2::Ident,
    generics: &'a syn::Generics,
    selections: Vec<Selection<'schema>>,
    variables_fields: syn::Type,
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
    schema_module_path: &'a syn::Path,
    field_marker_type_path: syn::Path,
    graphql_field_kind: FieldKind,
    graphql_field: &'a Field<'a>,
    arguments: super::arguments::Output<'a>,
    flatten: bool,
    alias: Option<String>,
    recurse_limit: Option<u8>,
    span: proc_macro2::Span,
    requires_feature: Option<String>,
    directives: Vec<AnalysedFieldDirective<'a>>,
}

struct SpreadSelection {
    rust_field_type: syn::Type,
    span: proc_macro2::Span,
}

impl<'schema, 'a: 'schema> FragmentImpl<'schema, 'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new_for(
        schema: &'a Schema<'schema, Unvalidated>,
        fields: &'a [(FragmentDeriveField, Option<Field<'schema>>)],
        name: &'a syn::Ident,
        generics: &'a syn::Generics,
        schema_type: &FragmentDeriveType<'schema>,
        schema_module_path: &'a syn::Path,
        graphql_type_name: &str,
        variables: Option<&syn::Path>,
    ) -> Result<Self, Errors> {
        let target_struct = name;

        let schema_type_path = schema_type.marker_ident.to_path(schema_module_path);

        let field_module_path = schema_type.field_module.to_path(schema_module_path);

        let variables_fields = variables_fields_path(variables);
        let variables_fields = variables_fields.as_ref();

        let selections = fields
            .iter()
            .map(|(field, schema_field)| {
                process_field(
                    schema,
                    field,
                    schema_field.as_ref(),
                    &field_module_path,
                    schema_module_path,
                    variables_fields,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;

        let variables_fields = if let Some(vars) = variables_fields {
            let span = vars.span();
            syn::parse2(quote_spanned! { span => #vars })?
        } else {
            syn::parse2(quote! { () })?
        };

        Ok(FragmentImpl {
            selections,
            target_struct,
            generics,
            variables_fields,
            graphql_type_name: graphql_type_name.to_string(),
            schema_type_path,
        })
    }
}

fn process_field<'a>(
    schema: &'a Schema<'a, Unvalidated>,
    field: &FragmentDeriveField,
    schema_field: Option<&'a Field<'a>>,
    field_module_path: &syn::Path,
    schema_module_path: &'a syn::Path,
    variables_fields: Option<&syn::Path>,
) -> Result<Selection<'a>, Errors> {
    let ty = &field.raw_field.ty;
    if field.type_check_mode() == CheckMode::Spreading {
        check_spread_type(ty)?;

        return Ok(Selection::Spread(SpreadSelection {
            rust_field_type: ty.clone(),
            span: ty.span(),
        }));
    }

    let schema_field = schema_field.expect("only spread fields should have schema_field == None");

    let (arguments, argument_span) = arguments_from_field_attrs(&field.raw_field.attrs)?
        .unwrap_or_else(|| (vec![], Span::call_site()));

    let arguments = process_arguments(
        schema,
        arguments,
        schema_field,
        schema_module_path.clone(),
        variables_fields,
        argument_span,
    )?;

    let mut directives = vec![];
    for directive in &field.directives {
        directives.push(process_directive(
            schema,
            directive.clone(),
            variables_fields,
            argument_span,
        )?);
    }

    check_types_are_compatible(
        &schema_field.field_type,
        &field.raw_field.ty,
        field.type_check_mode(),
    )?;

    let field_marker_type_path = schema_field.marker_ident().to_path(field_module_path);

    Ok(Selection::Field(FieldSelection {
        rust_field_type: field.raw_field.ty.clone(),
        arguments,
        schema_module_path,
        field_marker_type_path,
        graphql_field: schema_field,
        recurse_limit: field.raw_field.recurse.as_ref().map(|f| **f),
        span: ty.span(),
        alias: field.alias(),
        graphql_field_kind: schema_field.field_type.inner_type(schema).as_kind(),
        flatten: *field.raw_field.flatten,
        requires_feature: field
            .raw_field
            .feature
            .as_ref()
            .map(|feature| feature.as_ref().clone()),
        directives,
    }))
}

impl quote::ToTokens for FragmentImpl<'_, '_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::TokenStreamExt;

        let variables_fields = &self.variables_fields;
        let target_struct = &self.target_struct;
        let selections = &self.selections;
        let graphql_type = proc_macro2::Literal::string(&self.graphql_type_name);
        let schema_type = &self.schema_type_path;
        let fragment_name = proc_macro2::Literal::string(&target_struct.to_string());
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        tokens.append_all(quote! {
            #[automatically_derived]
            impl #impl_generics cynic::QueryFragment for #target_struct #ty_generics #where_clause {
                type SchemaType = #schema_type;
                type VariablesFields = #variables_fields;

                const TYPE: Option<&'static str> = Some(#graphql_type);

                fn query(mut builder: cynic::queries::SelectionBuilder<'_, Self::SchemaType, Self::VariablesFields>)
                where
                {
                    #![allow(unused_mut)]

                    #(#selections)*
                }

                fn name() -> Option<std::borrow::Cow<'static, str>> {
                    Some(std::borrow::Cow::Borrowed(#fragment_name))
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

        let directives = self
            .directives
            .iter()
            .map(|analysed| super::directives::Output {
                analysed,
                schema_module: self.schema_module_path,
            })
            .collect::<Vec<_>>();

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
                types::align_output_type(field_type, &self.graphql_field.field_type)
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
                    <#aligned_type as cynic::QueryFragment>::SchemaType
                }
            }
            FieldKind::Scalar if self.flatten => quote_spanned! { self.span =>
                <#aligned_type as cynic::schema::IsScalar<#aligned_type>>::SchemaType
            },
            FieldKind::Scalar => quote_spanned! { self.span =>
                <#aligned_type as cynic::schema::IsScalar<
                    <#field_marker_type_path as cynic::schema::Field>::Type
                >>::SchemaType
            },
            FieldKind::Enum => quote_spanned! { self.span =>
                <#aligned_type as cynic::Enum>::SchemaType
            },
        };

        let select_tokens = match selection_mode {
            SelectionMode::Composite => {
                quote_spanned! { self.span =>
                    let mut field_builder = builder
                        .select_field::<
                            #field_marker_type_path,
                            #schema_type_lookup
                        >();

                    #alias
                    #arguments
                    #(#directives)*

                    <#aligned_type as cynic::QueryFragment>::query(
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
                            <#field_marker_type_path as cynic::schema::Field>::Type,
                        >();

                    #alias
                    #arguments
                    #(#directives)*

                    <#aligned_type as cynic::QueryFragment>::query(
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
                            <#field_marker_type_path as cynic::schema::Field>::Type,
                        >();

                    #alias
                    #arguments
                    #(#directives)*
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
                        #(#directives)*

                        <#aligned_type as cynic::QueryFragment>::query(
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
                    #(#directives)*
                }
            }
        };

        match &self.requires_feature {
            Some(required_feature) => {
                let string_lit = proc_macro2::Literal::string(required_feature);
                tokens.append_all(quote! {
                    if builder.is_feature_enabled(#string_lit) {
                        #select_tokens
                    }
                })
            }
            _ => tokens.append_all(select_tokens),
        }
    }
}

impl quote::ToTokens for SpreadSelection {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::TokenStreamExt;
        let field_type = &self.rust_field_type;

        tokens.append_all(quote_spanned! { self.span =>
            <#field_type as cynic::QueryFragment>::query(
                builder
                    .inline_fragment()
                    .select_children::<<#field_type as cynic::QueryFragment>::VariablesFields>()
            );
        })
    }
}
