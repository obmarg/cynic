mod field_serializer;

use {
    proc_macro2::{Span, TokenStream},
    quote::quote,
};

use crate::{
    error::Errors,
    generics_for_serde,
    idents::RenameAll,
    input_object_derive::{
        InputObjectDeriveInput, input::InputObjectDeriveField, pairing::pair_fields,
        standard::field_serializer::FieldSerializer,
    },
    schema::{Schema, types::InputObjectType},
};

pub fn standard_input_object_derive(
    input: &InputObjectDeriveInput,
    struct_span: Span,
    schema: &Schema<'_, crate::schema::Unvalidated>,
    input_object: InputObjectType<'_>,
    rename_all: RenameAll,
    fields: &darling::ast::Fields<InputObjectDeriveField>,
) -> Result<TokenStream, Errors> {
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let generics_with_ser = generics_for_serde::with_serialize_bounds(&input.generics);
    let (impl_generics_with_ser, _, where_clause_with_ser) = generics_with_ser.split_for_impl();
    let input_marker_ident = input_object.marker_ident().to_rust_ident();
    let schema_module = input.schema_module();
    let graphql_type_name = proc_macro2::Literal::string(input_object.name.as_ref());

    let pairs = pair_fields(
        &fields.fields,
        input_object,
        rename_all,
        input.require_all_fields,
        &struct_span,
    )?;

    let field_serializers = pairs
        .into_iter()
        .map(|(rust_field, graphql_field)| {
            FieldSerializer::new(rust_field, graphql_field, &schema_module)
        })
        .collect::<Vec<_>>();

    let errors = field_serializers
        .iter()
        .filter_map(|fs| fs.validate())
        .collect::<Errors>();

    if !errors.is_empty() {
        return Ok(errors.to_compile_errors());
    }

    let typechecks = field_serializers
        .iter()
        .map(|fs| fs.type_check(&impl_generics, where_clause, schema));
    let map_serializer_ident = proc_macro2::Ident::new("map_serializer", Span::call_site());
    let field_inserts = field_serializers
        .iter()
        .map(|fs| fs.field_insert_call(&map_serializer_ident));

    let map_len = field_serializers.len();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics cynic::InputObject for #ident #ty_generics #where_clause_with_ser {
            type SchemaType = #schema_module::#input_marker_ident;
        }

        #[automatically_derived]
        impl #impl_generics_with_ser cynic::serde::Serialize for #ident #ty_generics #where_clause_with_ser {
            fn serialize<__S>(&self, serializer: __S) -> Result<__S::Ok, __S::Error>
            where
                __S: cynic::serde::Serializer,
            {
                use cynic::serde::ser::SerializeMap;
                #(#typechecks)*

                let mut map_serializer = serializer.serialize_map(Some(#map_len))?;

                #(#field_inserts)*

                map_serializer.end()
            }
        }

        cynic::impl_coercions!(#ident #ty_generics [#impl_generics] [#where_clause], #schema_module::#input_marker_ident);

        #[automatically_derived]
        impl #impl_generics #schema_module::variable::Variable for #ident #ty_generics #where_clause {
            const TYPE: cynic::variables::VariableType = cynic::variables::VariableType::Named(#graphql_type_name);
        }
    })
}
