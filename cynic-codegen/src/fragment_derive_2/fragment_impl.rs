use std::collections::{HashMap, HashSet};

use proc_macro2::{Span, TokenStream};
use syn::spanned::Spanned;

use crate::{
    idents::PathExt,
    schema::{
        names::FieldName,
        types::{Field, OutputType, TypeRef},
    },
    type_validation_2::{check_spread_type, check_types_are_compatible, CheckMode},
    Ident,
};

use super::arguments::{arguments_from_field_attrs, FieldArgument};
use super::fragment_derive_type::FragmentDeriveType;

use super::input::{FragmentDeriveField, FragmentDeriveInput};

use crate::suggestions::{format_guess, guess_field};

pub struct FragmentImpl {
    target_struct: Ident,
    field_selections: Vec<FieldSelection>,
    argument_struct: syn::Type,
    graphql_type_name: String,
    schema_type_path: syn::Path,
}

struct FieldSelection {
    // graphql_field_ident: Ident,
    rust_field_ident: proc_macro2::Ident,
    rust_field_type: syn::Type,
    field_marker_type_path: syn::Path,
    graphql_field_kind: FieldKind,
    arguments: Vec<FieldArgument>,
    // recurse_limit: Option<u8>,
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
        fields: &[(&FragmentDeriveField, &Field<'_>)],
        name: &syn::Ident,
        schema_type: &FragmentDeriveType,
        schema_module_path: &syn::Path,
        graphql_type_name: &str,
        argument_struct: syn::Type,
    ) -> Result<Self, syn::Error> {
        let target_struct = Ident::new_spanned(&name.to_string(), name.span());

        let schema_type_path = schema_type.marker_ident.located_at_path(schema_module_path);

        let field_module_path = schema_type.field_module.located_at_path(schema_module_path);

        let field_selections = fields
            .iter()
            .map(|(field, schema_field)| {
                process_field(field, schema_field, &field_module_path, graphql_type_name)
            })
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
    schema_field: &Field<'_>,
    field_module_path: &syn::Path,
    graphql_type_name: &str,
) -> Result<FieldSelection, syn::Error> {
    // Should be safe to unwrap because we've already checked we have a struct
    // style input
    let field_ident = field
        .ident
        .as_ref()
        .expect("Fragment derive only supports named structs");

    let graphql_ident = field.graphql_ident();

    let field_name_span = graphql_ident.span();

    let arguments = arguments_from_field_attrs(&field.attrs)?;

    if field.type_check_mode() == CheckMode::Spreading {
        check_spread_type(&field.ty)?;

        // let mut field_marker_type_path = field_module_path;
        // field_marker_type_path.push(graphql_ident.as_field_marker_type());

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
        todo!("spreading not implemented")
        // TODO: Return
    }

    check_types_are_compatible(&schema_field.field_type, &field.ty, field.type_check_mode())?;

    validate_args(&arguments, schema_field, field_name_span)?;

    let field_marker_type_path = schema_field
        .marker_ident()
        .located_at_path(field_module_path);

    let field_selection = FieldSelection {
        // graphql_field_ident: graphql_ident.clone(),
        rust_field_ident: field_ident.clone(),
        rust_field_type: field.ty.clone(),
        arguments,
        field_marker_type_path,
        // recurse_limit: field.recurse.as_ref().map(|f| **f),
        span: field.ty.span(),
        graphql_field_kind: schema_field.field_type.inner_type().as_kind(),
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

    // TODO: MOve this into pair_fields
    // } else {
    //     let candidates = schema_type.fields.iter().map(|f| f.name.as_str());
    //     let graphql_name = graphql_ident.graphql_name();
    //     let guess_value = guess_field(candidates, &graphql_name);
    //     Err(syn::Error::new(
    //         field_name_span,
    //         format!(
    //             "Field {} does not exist on the GraphQL type {}.{}",
    //             graphql_name,
    //             graphql_type_name,
    //             format_guess(guess_value).as_str()
    //         ),
    //     ))
    // }
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
                todo!("need to handle interface type fields")
            }
            FieldKind::Union => {
                // TODO: Not sure this is right, but figure it out....
                // If it is might be able to merge w/ object
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
                }
            }
        });
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

/// Validates the FieldArguments against the arguments defined on field in the schema.
fn validate_args(
    arguments: &[FieldArgument],
    field: &Field,
    missing_arg_span: Span,
) -> Result<(), syn::Error> {
    let all_required = field
        .arguments
        .iter()
        .filter(|arg| !matches!(arg.value_type, TypeRef::Nullable(_)))
        .map(|arg| arg.name.as_str().to_string())
        .collect::<HashSet<_>>();

    let provided_names = arguments
        .iter()
        .map(|arg| arg.argument_name.to_string())
        .collect::<HashSet<_>>();

    let missing_args = all_required
        .difference(&provided_names)
        .cloned()
        .collect::<Vec<_>>();

    if !missing_args.is_empty() {
        let missing_args = missing_args.join(", ");

        return Err(syn::Error::new(
            missing_arg_span,
            format!("Missing arguments: {}", missing_args),
        ));
    }

    let all_arg_names = field
        .arguments
        .iter()
        .map(|arg| arg.name.as_str().to_string())
        .collect::<HashSet<_>>();

    let unknown_args = provided_names
        .difference(&all_arg_names)
        .cloned()
        .collect::<Vec<_>>();

    if !unknown_args.is_empty() {
        let unknown_args = unknown_args.join(", ");

        return Err(syn::Error::new(
            missing_arg_span,
            format!("Unknown arguments: {}", unknown_args),
        ));
    }

    Ok(())
}
