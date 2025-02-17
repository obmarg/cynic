use proc_macro2::TokenStream;
use quote::quote_spanned;

use crate::schema::{Schema, Unvalidated};

use super::FieldKind;

use {
    super::FragmentDeriveField,
    crate::{generics_for_serde, schema::types as schema},
};

pub enum DeserializeImpl<'a> {
    Standard(StandardDeserializeImpl<'a>),
    Spreading(SpreadingDeserializeImpl<'a>),
}

pub struct StandardDeserializeImpl<'a> {
    target_struct: &'a syn::Ident,
    fields: Vec<Field>,
    generics: &'a syn::Generics,
}

pub struct SpreadingDeserializeImpl<'a> {
    target_struct: &'a syn::Ident,
    fields: Vec<Field>,
    generics: &'a syn::Generics,
}

struct Field {
    rust_name: proc_macro2::Ident,
    ty: syn::Type,
    field_variant_name: proc_macro2::Ident,
    serialized_name: Option<String>,
    inner_kind: Option<FieldKind>,
    field_marker: syn::Path,
    is_spread: bool,
    is_flattened: bool,
    is_recurse: bool,
    is_feature_flagged: bool,
    is_skippable: bool,
}

impl<'a> DeserializeImpl<'a> {
    pub fn new(
        schema: &Schema<'_, Unvalidated>,
        fields: &[(FragmentDeriveField, Option<schema::Field<'_>>)],
        name: &'a syn::Ident,
        generics: &'a syn::Generics,
        field_module_path: &syn::Path,
    ) -> Self {
        let spreading = fields.iter().any(|f| f.0.spread());

        let target_struct = name;
        let fields = fields
            .iter()
            .map(|(field, schema_field)| {
                process_field(schema, field, schema_field.as_ref(), field_module_path)
            })
            .collect();

        match spreading {
            true => DeserializeImpl::Spreading(SpreadingDeserializeImpl {
                target_struct,
                fields,
                generics,
            }),
            false => DeserializeImpl::Standard(StandardDeserializeImpl {
                target_struct,
                fields,
                generics,
            }),
        }
    }
}

impl quote::ToTokens for DeserializeImpl<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            DeserializeImpl::Standard(inner) => inner.to_tokens(tokens),
            DeserializeImpl::Spreading(inner) => inner.to_tokens(tokens),
        }
    }
}

impl quote::ToTokens for StandardDeserializeImpl<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let target_struct = &self.target_struct;
        let serialized_names = self
            .fields
            .iter()
            .map(|f| {
                proc_macro2::Literal::string(
                    f.serialized_name
                        .as_deref()
                        .expect("non-spread fields must have a serialized_name"),
                )
            })
            .collect::<Vec<_>>();
        let field_variant_names = self
            .fields
            .iter()
            .map(|f| &f.field_variant_name)
            .collect::<Vec<_>>();
        let field_names = self.fields.iter().map(|f| &f.rust_name).collect::<Vec<_>>();
        let field_decodes = self.fields.iter().map(|f| {
            let field_name = &f.rust_name;
            let field_marker = &f.field_marker;
            let ty = &f.ty;
            let mut ty = quote! { #ty };
            let mut trailer = quote! {};

            if matches!(f.inner_kind, Some(FieldKind::Scalar)) {
                ty = quote! { cynic::__private::ScalarDeserialize<#ty, <#field_marker as cynic::schema::Field>::Type> };
                trailer.append_all(quote! { .into_inner() });
            }

            if f.is_flattened {
                ty = quote! { cynic::__private::Flattened<#ty> };
                trailer.append_all(quote! { .into_inner() });
            }

            quote! {
                #field_name = Some(__map.next_value::<#ty>()? #trailer);
            }
        });

        let struct_name = self.target_struct.to_string();
        let expecting_str = proc_macro2::Literal::string(&format!("struct {}", &struct_name));
        let struct_name = proc_macro2::Literal::string(&struct_name);

        let (_, ty_generics, _) = self.generics.split_for_impl();
        let generics_with_de = generics_for_serde::with_de_and_deserialize_bounds(self.generics);
        let (impl_generics, ty_generics_with_de, where_clause) = generics_with_de.split_for_impl();

        let field_unwraps = self.fields.iter().zip(&serialized_names).map(|(field, serialized_name)| {
            let rust_name = &field.rust_name;
            if field.is_recurse || field.is_feature_flagged {
                let span = rust_name.span();
                quote_spanned!{ span =>
                    let #rust_name = #rust_name.unwrap_or_default();
                }
            } else if field.is_skippable {
                quote! {
                    let #rust_name = #rust_name.unwrap_or_default();
                }
            } else {
                quote! {
                    let #rust_name = #rust_name.ok_or_else(|| cynic::serde::de::Error::missing_field(#serialized_name))?;
                }

            }
        }).collect::<Vec<_>>();

        tokens.append_all(quote! {
            #[automatically_derived]
            impl #impl_generics cynic::serde::Deserialize<'de> for #target_struct #ty_generics #where_clause {
                fn deserialize<__D>(deserializer: __D) -> Result<Self, __D::Error>
                where
                    __D: cynic::serde::Deserializer<'de>,
                {
                    #[derive(cynic::serde::Deserialize)]
                    #[serde(field_identifier, crate="cynic::serde")]
                    #[allow(non_camel_case_types)]
                    enum __FragmentDeriveField {
                        #(
                            #[serde(rename = #serialized_names)]
                            #field_variant_names,
                        )*
                        #[serde(other)]
                        __Other
                    }

                    struct Visitor #generics_with_de #where_clause {
                        marker: ::core::marker::PhantomData<#target_struct #ty_generics>,
                        lifetime: ::core::marker::PhantomData<&'de ()>,
                    }

                    impl #impl_generics cynic::serde::de::Visitor<'de> for Visitor #ty_generics_with_de #where_clause {
                        type Value = #target_struct #ty_generics;

                        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                            formatter.write_str(#expecting_str)
                        }

                        fn visit_map<V>(self, mut __map: V) -> Result<Self::Value, V::Error>
                        where
                            V: cynic::serde::de::MapAccess<'de>,
                        {
                            #(
                                let mut #field_names = None;
                            )*
                            while let Some(__key) = __map.next_key()? {
                                match __key {
                                    #(
                                        __FragmentDeriveField::#field_variant_names => {
                                            if #field_names.is_some() {
                                                return Err(cynic::serde::de::Error::duplicate_field(#serialized_names));
                                            }
                                            #field_decodes
                                        }
                                    )*
                                    __FragmentDeriveField::__Other => {
                                        __map.next_value::<cynic::serde::de::IgnoredAny>()?;
                                    }
                                }
                            }
                            #(#field_unwraps)*
                            Ok(#target_struct {
                                #(#field_names),*
                            })
                        }
                    }

                    const FIELDS: &'static [&str] = &[#(#serialized_names),*];

                    deserializer.deserialize_struct(
                        #struct_name,
                        FIELDS,
                        Visitor {
                            marker: ::core::marker::PhantomData,
                            lifetime: ::core::marker::PhantomData,
                        },
                    )
                }
            }
        });
    }
}

impl quote::ToTokens for SpreadingDeserializeImpl<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let target_struct = &self.target_struct;

        let field_inserts = self.fields.iter().map(|f| {
            let field_name = &f.rust_name;
            let field_ty = &f.ty;
            if f.is_spread {
                quote! {
                    #field_name: <#field_ty as cynic::serde::Deserialize<'de>>::deserialize(
                        spreadable.spread_deserializer()
                    )?
                }
            } else if f.is_flattened {
                let serialized_name = proc_macro2::Literal::string(
                    f.serialized_name
                        .as_deref()
                        .expect("non spread fields must have a serialized_name"),
                );
                quote! {
                    #field_name: spreadable.deserialize_field::<
                        cynic::__private::Flattened<#field_ty>
                    >(#serialized_name)?.into_inner()
                }
            } else {
                let serialized_name = proc_macro2::Literal::string(
                    f.serialized_name
                        .as_deref()
                        .expect("non spread fields must have a serialized_name"),
                );
                quote! {
                    #field_name: spreadable.deserialize_field(#serialized_name)?
                }
            }
        });

        let (_, ty_generics, where_clause) = self.generics.split_for_impl();
        let generics_with_de = generics_for_serde::with_de_and_deserialize_bounds(self.generics);
        let (impl_generics, _, _) = generics_with_de.split_for_impl();

        tokens.append_all(quote! {
            #[automatically_derived]
            impl #impl_generics cynic::serde::Deserialize<'de> for #target_struct #ty_generics #where_clause {
                fn deserialize<__D>(deserializer: __D) -> Result<Self, __D::Error>
                where
                    __D: cynic::serde::Deserializer<'de>,
                {
                    let spreadable = cynic::__private::Spreadable::<__D::Error>::deserialize(deserializer)?;

                    Ok(#target_struct {
                        #(#field_inserts,)*
                    })
                }
            }
        });
    }
}

fn process_field(
    schema: &Schema<'_, Unvalidated>,
    field: &FragmentDeriveField,
    schema_field: Option<&schema::Field<'_>>,
    field_module_path: &syn::Path,
) -> Field {
    // Should be ok to unwrap since we only accept struct style input
    let rust_name = field.ident().unwrap();
    let field_variant_name = rust_name.clone();
    let schema_type = schema_field.map(|field| field.field_type.inner_type(schema));
    let inner_kind = schema_type.as_ref().map(|ty| ty.as_kind());

    let field_marker = match schema_field {
        Some(field) => field.marker_ident().to_path(field_module_path),
        None => syn::parse_quote!(String),
    };

    Field {
        field_variant_name,
        serialized_name: field
            .alias()
            .or_else(|| schema_field.map(|f| f.name.as_str().to_string())),
        rust_name: rust_name.clone(),
        ty: field.raw_field.ty.clone(),
        is_spread: field.spread(),
        is_flattened: *field.raw_field.flatten,
        is_recurse: field.raw_field.recurse.is_some(),
        is_feature_flagged: field.raw_field.feature.is_some(),
        is_skippable: field.is_skippable(),
        inner_kind,
        field_marker,
    }
}
