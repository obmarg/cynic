use std::collections::{HashMap, HashSet};

use proc_macro2::{Span, TokenStream};
use syn::spanned::Spanned;

use crate::{
    ident::PathExt,
    load_schema,
    type_validation::{check_spread_type, check_types_are_compatible, CheckMode},
    Errors, FieldType, Ident,
};

use super::arguments::{arguments_from_field_attrs, FieldArgument};
use super::schema_parsing::{Field, Object};
use super::type_ext::SynTypeExt;

use super::input::{FragmentDeriveField, FragmentDeriveInput};

use crate::suggestions::{format_guess, guess_field};

pub struct FragmentImpl {
    target_struct: Ident,
    field_selections: Vec<FieldSelection>,
    // TODO: Whatever we need to write the deserialize impl
    // fields: Vec<FieldSelectorCall>,
    // selector_struct_path: syn::Path,
    // constructor_params: Vec<ConstructorParameter>,
    argument_struct: syn::Type,
    graphql_type_name: String,
    schema_type_path: syn::Path,
}

struct FieldSelection {
    graphql_field_ident: Ident,
    rust_field_ident: proc_macro2::Ident,
    rust_field_type: syn::Type,
    field_marker_type_path: syn::Path,
    graphql_field_kind: FieldKind,
    arguments: Vec<FieldArgument>,
    recurse_limit: Option<u8>,
    span: proc_macro2::Span,
}

enum FieldKind {
    Composite,
    Scalar,
    Enum,

    // TODO: how to handle these two?  Presumably similar technique to scalars/enums?
    Interface,
    Union,
}

impl FragmentImpl {
    pub fn new_for(
        fields: &darling::ast::Fields<FragmentDeriveField>,
        name: &syn::Ident,
        object: &Object,
        schema_module_path: syn::Path,
        graphql_type_name: &str,
        argument_struct: syn::Type,
    ) -> Result<Self, syn::Error> {
        let target_struct = Ident::new_spanned(&name.to_string(), name.span());

        let mut schema_type_path = schema_module_path.clone();
        schema_type_path.push(&object.rust_type_name);

        let mut field_module_path = schema_module_path;
        field_module_path.push(object.rust_type_name.as_field_module());

        let field_selections = fields
            .fields
            .iter()
            .map(|field| process_field(field, object, field_module_path.clone(), graphql_type_name))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(FragmentImpl {
            field_selections,
            target_struct,
            argument_struct,
            graphql_type_name: graphql_type_name.to_string(),
            schema_type_path,
        })
    }
}

fn process_field(
    field: &FragmentDeriveField,
    object: &Object,
    field_module_path: syn::Path,
    graphql_type_name: &str,
) -> Result<FieldSelection, syn::Error> {
    // Should be safe to unwrap because we've already checked we have a struct
    // style input
    let (field_ident, ref graphql_ident) = field.ident.as_ref().zip(field.graphql_ident()).unwrap();

    let field_name_span = graphql_ident.span();

    let arguments = arguments_from_field_attrs(&field.attrs)?;

    if field.type_check_mode() == CheckMode::Spreading {
        check_spread_type(&field.ty)?;

        let mut field_marker_type_path = field_module_path;
        field_marker_type_path.push(graphql_ident.as_field_marker_type());

        /*
        let field_selection = FieldSelection {
            graphql_field_ident: graphql_ident.clone(),
            rust_field_ident: field_ident.clone(),
            rust_field_type: field.ty.clone(),
            arguments,
            field_marker_type_path,
            recurse_limit: None,
            span: field.ty.span(),
        };

        Ok(field_selection)
        */
        todo!()
    } else if let Some(gql_field) = object.fields.get(graphql_ident) {
        check_types_are_compatible(&gql_field.field_type, &field.ty, field.type_check_mode())?;

        validate_args(&arguments, gql_field, field_name_span)?;

        let mut field_marker_type_path = field_module_path;
        field_marker_type_path.push(graphql_ident.as_field_marker_type());

        let field_selection = FieldSelection {
            graphql_field_ident: graphql_ident.clone(),
            rust_field_ident: field_ident.clone(),
            rust_field_type: field.ty.clone(),
            arguments,
            field_marker_type_path,
            recurse_limit: field.recurse.as_ref().map(|f| **f),
            span: field.ty.span(),
            graphql_field_kind: gql_field.field_type.as_kind(),
        };
        /*
        let field_selector = FieldSelectorCall {
            selector_function: FieldTypeSelectorCall::for_field(
                &gql_field.field_type,
                field_constructor,
                *field.flatten,
                field.recurse.as_ref().map(|f| **f),
                field.alias(),
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
        }; */

        Ok(field_selection)
    } else {
        let candidates = object.fields.keys().map(|k| k.graphql_name());
        let graphql_name = graphql_ident.graphql_name();
        let guess_value = guess_field(candidates, graphql_name);
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
        let field_selections = &self.field_selections;
        let graphql_type = proc_macro2::Literal::string(&self.graphql_type_name);
        let schema_type = &self.schema_type_path;

        tokens.append_all(quote! {
            #[automatically_derived]
            impl<'de> ::cynic::core::QueryFragment<'de> for #target_struct {
                type SchemaType = #schema_type;

                fn query(mut builder: ::cynic::queries::QueryBuilder<Self::SchemaType>) {
                    #(#field_selections)*
                }
            }

            // TODO: The deserialize impl
        })
    }
}

impl quote::ToTokens for FieldSelection {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote_spanned, TokenStreamExt};

        let field_marker_type_path = &self.field_marker_type_path;
        let field_name = &self.rust_field_ident; // TODO: Pascal case this.
        let field_type = &self.rust_field_type;

        tokens.append_all(match self.graphql_field_kind {
            FieldKind::Composite => {
                quote_spanned! { self.span =>
                    let mut field_builder = builder
                        .select_field::<
                            #field_marker_type_path,
                            <#field_type as ::cynic::core::QueryFragment>::SchemaType
                        >();

                    // TODO: Arguments

                    <#field_type as ::cynic::core::QueryFragment>::query(
                        field_builder.select_children()
                    );

                    field_builder.done();
                }
            }
            FieldKind::Enum => {
                quote_spanned! { self.span =>
                    let mut field_builder = builder
                        .select_field::<
                            #field_marker_type_path,
                            <#field_type as ::cynic::schema::IsEnum<
                                <#field_marker_type_path as ::cynic::schema::Field>::SchemaType
                            >>::SchemaType
                        >();

                    field_builder.done();
                }
            }
            FieldKind::Scalar => {
                quote_spanned! { self.span =>
                    let mut field_builder = builder
                        .select_field::<
                            #field_marker_type_path,
                            <#field_type as ::cynic::schema::IsScalar<
                                <#field_marker_type_path as ::cynic::schema::Field>::SchemaType
                            >>::SchemaType
                        >();

                    field_builder.done();
                }
            }
            FieldKind::Interface => {
                todo!()
            }
            FieldKind::Union => {
                todo!()
            }
        });
    }
}

impl FieldType {
    fn as_kind(&self) -> FieldKind {
        match self {
            FieldType::Scalar(_, _) | FieldType::BuiltInScalar(_, _) => FieldKind::Scalar,
            FieldType::Enum(_, _) => FieldKind::Enum,
            // TODO: handle the other FieldKind(s)
            _ => FieldKind::Composite,
        }
    }
}

/// Validates the FieldArguments against the arguments defined on field in the schema.
fn validate_args(
    arguments: &[FieldArgument],
    field: &Field,
    missing_arg_span: Span,
) -> Result<(), syn::Error> {
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

    Ok(())
}
