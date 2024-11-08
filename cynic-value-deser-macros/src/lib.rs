use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse_quote, spanned::Spanned, Fields, ImplGenerics};

#[proc_macro_derive(ValueDeserialize, attributes(deser))]
pub fn value_deserialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // TODO: Detect if this is autocomplete etc.

    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    match value_deser_impl(ast) {
        Ok(tokens) => tokens.into(),
        Err(e) => {
            todo!("put out a dummy impl as well as the compile errors");
            // e.to_compile_error().into(),
        }
    }
}

fn value_deser_impl(ast: syn::DeriveInput) -> Result<TokenStream, ()> {
    let syn::Data::Struct(data) = ast.data else {
        panic!("ValueDeserialize can only be derived on structs");
    };

    let ident = ast.ident;

    let (original_impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();
    let (impl_generics, deser_lifetime) = match ast.generics.lifetimes().next() {
        Some(lifetime) => {
            let mut lifetime = lifetime.clone();
            lifetime.attrs = vec![];
            lifetime.bounds = Default::default();
            lifetime.colon_token = None;
            (original_impl_generics.to_token_stream(), lifetime)
        }
        None => {
            let mut generics = ast.generics.clone();
            generics.params.push(parse_quote!('a));
            let (impl_generics, ..) = generics.split_for_impl();
            (impl_generics.to_token_stream(), parse_quote!('a))
        }
    };

    let Fields::Named(named) = data.fields else {
        panic!("ValueDeserialize can only be derived on structs with named fields");
    };

    let field_names = named
        .named
        .iter()
        .map(|f| f.ident.clone().unwrap())
        .collect::<Vec<_>>();
    let field_name_strings = field_names
        .iter()
        .map(|name| proc_macro2::Literal::string(&name.to_string()))
        .collect::<Vec<_>>();
    let field_decodes = named
        .named
        .iter()
        .map(|field| {
            let field_name = &field.ident;
            let ty = &field.ty;
            quote! {
                #field_name = Some(field.value().deserialize()?);
            }
        })
        .collect::<Vec<_>>();

    let field_unwraps = named
        .named
        .iter()
        .map(|field| {
            let field_name = field.ident.as_ref().unwrap();
            let field_name_string = proc_macro2::Literal::string(&field_name.to_string());
            quote! {
                let #field_name = #field_name.ok_or_else(|| cynic_value_deser::Error::MissingField {
                    name: #field_name_string,
                    object_span: input.span()
                })?;
            }
        })
        .collect::<Vec<_>>();

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics cynic_value_deser::ValueDeserialize #deser_lifetime for #ident #ty_generics #where_clause {
            fn deserialize(input: cynic_value_deser::DeserValue #deser_lifetime) -> Result<Self, cynic_value_deser::Error> {
                use cynic_value_deser::ConstDeserializer;
                let cynic_value_deser::DeserValue::Object(obj) = input else {
                    return Err(cynic_value_deser::Error::unexpected_type(
                        cynic_value_deser::value::ValueType::Object,
                        input
                    ));
                };
                #(
                    let mut #field_names = None;
                )*
                for field in obj.fields() {
                    match field.name() {
                        #(
                            #field_name_strings => {
                                if #field_names.is_some() {
                                    return Err(todo!("this error"))
                                }
                                #field_decodes
                            },
                        )*
                    }
                }
                #(#field_unwraps)*
                Ok(#ident {
                    #(#field_names),*
                })
            }
        }
    })
}
