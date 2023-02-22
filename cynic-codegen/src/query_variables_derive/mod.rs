use {
    proc_macro2::TokenStream,
    quote::{format_ident, quote, quote_spanned},
    syn::visit_mut::{self, VisitMut},
};

mod input;

use crate::variables_fields_ident;

use self::input::QueryVariablesDeriveInput;

pub fn query_variables_derive(ast: &syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    use darling::FromDeriveInput;

    match QueryVariablesDeriveInput::from_derive_input(ast) {
        Ok(input) => query_variables_derive_impl(input),
        Err(e) => Ok(e.write_errors()),
    }
}

pub fn query_variables_derive_impl(
    input: QueryVariablesDeriveInput,
) -> Result<TokenStream, syn::Error> {
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let vis = &input.vis;
    let schema_module = &input.schema_module();
    let fields_struct_ident = variables_fields_ident(ident);

    let input_fields = input.data.take_struct().unwrap().fields;

    let mut field_funcs = Vec::new();
    let mut variables = Vec::new();
    let mut field_inserts = Vec::new();
    let mut coercion_checks = Vec::new();
    let mut field_output_types = Vec::new();

    for (field_idx, f) in input_fields.into_iter().enumerate() {
        let name = f.ident.as_ref().unwrap();
        let ty = &f.ty;
        let mut ty_for_fields_struct = match f.graphql_type {
            None => ty.clone(),
            Some(ref graphql_type) => {
                // This enables to support generics. Normally we have every Variable type that
                // implements `CoercesTo<schema::SchemaType>`. Here that is also
                // the case, but:
                // - The XxxVariablesFields struct needs to not be generic, as it needs to be
                //   referenced explicitly by the `QueryFragment` type (to have access to the
                //   functions that enable to typecheck for the presence of variables), and we
                //   like the QueryFragment itself to not be generic on the variables type to
                //   generate the query.
                // - Normally we use the type of the fields of the XxxVariables directly in
                //   these return types of the functions of XxxVariablesFields, and that type
                //   implements `CoercesTo<schema::CorrespondingType>` (as derived by
                //   `InputObject`, or specified by cynic if it's a litteral)
                // - Now we want to still typecheck that our concrete (specialized) types have
                //   all their fields implement `CoercesTo` the correct schema type, but we also
                //   want to not be generic in the `Fields` struct. This is achieved by:
                //   - Creating a proxy type that will be used in the XxxVariablesFields struct
                //   - Implementing `CoercesTo<SchemaType>` for it (so that the regular
                //     typecheck passes)
                //   - Adding a separate typecheck in the concrete type for the
                //     `CoercesTo<schema_type>` (so that we still check for correctness)

                let new_type_that_coerces_to_schema_type =
                    format_ident!("CoercionProxyForField{field_idx}");
                field_output_types.push(quote! {
                    #vis struct #new_type_that_coerces_to_schema_type;
                    ::cynic::impl_coercions!(#new_type_that_coerces_to_schema_type [] [], #schema_module::#graphql_type);
                });
                coercion_checks.push(quote! {
                    ::cynic::assert_impl!(#ty [#impl_generics] [#where_clause]: ::cynic::coercions::CoercesTo<#schema_module::#graphql_type>);
                });

                // Turn that from an ident into a type
                syn::parse_quote! { #new_type_that_coerces_to_schema_type }
            }
        };
        TurnLifetimesToStatic.visit_type_mut(&mut ty_for_fields_struct);
        let name_str =
            proc_macro2::Literal::string(&f.graphql_ident(input.rename_all).graphql_name());

        field_funcs.push(quote! {
            #vis fn #name() -> ::cynic::variables::VariableDefinition<Self, #ty_for_fields_struct> {
                ::cynic::variables::VariableDefinition::new(#name_str)
            }
        });

        variables.push(quote! {
            (#name_str, <#ty as #schema_module::variable::Variable>::TYPE)
        });

        field_inserts.push(quote! {
            map_serializer.serialize_entry(#name_str, &self.#name)?;
        })
    }

    let map_len = field_inserts.len();

    let ident_span = ident.span();
    let fields_struct = quote_spanned! { ident_span =>
        #vis struct #fields_struct_ident;

        impl ::cynic::QueryVariablesFields for #fields_struct_ident {}

        impl ::cynic::queries::VariableMatch<#fields_struct_ident> for #fields_struct_ident {}

        const _: () = {
            #(
                #field_output_types
            )*

            impl #fields_struct_ident {
                #(
                    #field_funcs
                )*
            }
        };
    };

    Ok(quote! {

        #[automatically_derived]
        impl #impl_generics ::cynic::QueryVariables for #ident #ty_generics #where_clause {
            type Fields = #fields_struct_ident;
            const VARIABLES: &'static [(&'static str, ::cynic::variables::VariableType)]
                = &[#(#variables),*];
        }

        #[automatically_derived]
        impl #impl_generics ::cynic::serde::Serialize for #ident #ty_generics #where_clause {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::cynic::serde::Serializer,
            {
                use ::cynic::serde::ser::SerializeMap;
                #(#coercion_checks)*

                let mut map_serializer = serializer.serialize_map(Some(#map_len))?;

                #(#field_inserts)*

                map_serializer.end()
            }
        }

        #fields_struct
    })
}

struct TurnLifetimesToStatic;
impl VisitMut for TurnLifetimesToStatic {
    fn visit_lifetime_mut(&mut self, i: &mut syn::Lifetime) {
        i.ident = format_ident!("static");
        visit_mut::visit_lifetime_mut(self, i)
    }
}
